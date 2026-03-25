use rusqlite::params;
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<rusqlite::Connection>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CourseRow {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub author_github: Option<String>,
    pub description: String,
    pub language_id: String,
    pub language_display: String,
    pub license: Option<String>,
    pub lessons: i64,
    pub exercises: i64,
    pub has_stages: bool,
    pub platform: Option<String>,
    pub provision: String,
    pub tags: String, // JSON array
    pub estimated_hours: Option<f64>,
    pub checksum: String,
    pub published_at: String,
    pub min_learnlocal_version: Option<String>,
    pub status: String,
    pub package_filename: String,
    pub downloads: i64,
    pub created_at: String,
    // Provenance
    pub owner_github: String,
    pub forked_from_id: Option<String>,
    pub forked_from_version: Option<String>,
    pub forked_from_author: Option<String>,
    // Aggregated
    #[serde(skip)]
    pub avg_rating: Option<f64>,
    #[serde(skip)]
    pub review_count: i64,
}

pub struct NewCourse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub author_github: Option<String>,
    pub description: String,
    pub language_id: String,
    pub language_display: String,
    pub license: Option<String>,
    pub lessons: i64,
    pub exercises: i64,
    pub has_stages: bool,
    pub platform: Option<String>,
    pub provision: String,
    pub tags: String,
    pub estimated_hours: Option<f64>,
    pub checksum: String,
    pub min_learnlocal_version: Option<String>,
    pub package_filename: String,
    // Provenance
    pub owner_github: String,
    pub forked_from_id: Option<String>,
    pub forked_from_version: Option<String>,
    pub forked_from_author: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RatingsSummary {
    pub average: f64,
    pub count: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReviewRow {
    pub github_user: String,
    pub body: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
pub struct VersionRow {
    pub version: String,
    pub author: String,
    pub published_at: String,
    pub status: String,
    pub lessons: i64,
    pub exercises: i64,
}

impl Database {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> anyhow::Result<Self> {
        let conn = rusqlite::Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn init_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS courses (
                id TEXT NOT NULL,
                version TEXT NOT NULL,
                name TEXT NOT NULL,
                author TEXT NOT NULL,
                author_github TEXT,
                description TEXT NOT NULL,
                language_id TEXT NOT NULL,
                language_display TEXT NOT NULL,
                license TEXT,
                lessons INTEGER NOT NULL,
                exercises INTEGER NOT NULL,
                has_stages BOOLEAN DEFAULT FALSE,
                platform TEXT,
                provision TEXT DEFAULT 'system',
                tags TEXT DEFAULT '[]',
                estimated_hours REAL,
                checksum TEXT NOT NULL,
                published_at TEXT NOT NULL,
                min_learnlocal_version TEXT,
                status TEXT DEFAULT 'pending',
                package_filename TEXT NOT NULL,
                downloads INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                -- Provenance: who owns this course ID
                owner_github TEXT NOT NULL,
                -- Fork lineage: where this course was derived from
                forked_from_id TEXT,
                forked_from_version TEXT,
                forked_from_author TEXT,
                PRIMARY KEY (id, version)
            );

            CREATE TABLE IF NOT EXISTS ratings (
                course_id TEXT NOT NULL,
                github_user TEXT NOT NULL,
                stars INTEGER NOT NULL CHECK(stars BETWEEN 1 AND 5),
                created_at TEXT NOT NULL,
                UNIQUE(course_id, github_user)
            );

            CREATE TABLE IF NOT EXISTS reviews (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                course_id TEXT NOT NULL,
                github_user TEXT NOT NULL,
                body TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(course_id, github_user)
            );

            CREATE INDEX IF NOT EXISTS idx_ratings_course ON ratings(course_id);
            CREATE INDEX IF NOT EXISTS idx_reviews_course ON reviews(course_id);
            CREATE INDEX IF NOT EXISTS idx_courses_owner ON courses(owner_github);
            ",
        )?;
        Ok(())
    }

    /// List courses — only the latest approved version per course ID.
    pub fn list_courses(&self, include_pending: bool) -> anyhow::Result<Vec<CourseRow>> {
        let conn = self.conn.lock().unwrap();
        // Use a window function to get only the latest version per course ID
        let status_filter = if include_pending {
            "1=1"
        } else {
            "status = 'approved'"
        };
        let sql = format!(
            "WITH latest AS (
                SELECT *, ROW_NUMBER() OVER (PARTITION BY id ORDER BY created_at DESC) as rn
                FROM courses
                WHERE {status_filter}
            )
            SELECT l.*, COALESCE(r.avg_rating, 0) as avg_rating, COALESCE(rv.cnt, 0) as review_count
            FROM latest l
            LEFT JOIN (SELECT course_id, AVG(CAST(stars AS REAL)) as avg_rating FROM ratings GROUP BY course_id) r ON r.course_id = l.id
            LEFT JOIN (SELECT course_id, COUNT(*) as cnt FROM reviews GROUP BY course_id) rv ON rv.course_id = l.id
            WHERE l.rn = 1
            ORDER BY l.name"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], Self::map_course_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get a specific course (latest version or specific version).
    pub fn get_course(&self, id: &str) -> anyhow::Result<Option<CourseRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT c.*, COALESCE(r.avg_rating, 0) as avg_rating, COALESCE(rv.cnt, 0) as review_count
             FROM courses c
             LEFT JOIN (SELECT course_id, AVG(CAST(stars AS REAL)) as avg_rating FROM ratings GROUP BY course_id) r ON r.course_id = c.id
             LEFT JOIN (SELECT course_id, COUNT(*) as cnt FROM reviews GROUP BY course_id) rv ON rv.course_id = c.id
             WHERE c.id = ?1
             ORDER BY c.created_at DESC
             LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::map_course_row)?;
        match rows.next() {
            Some(Ok(row)) => Ok(Some(row)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Get all versions of a course (for version history display).
    #[allow(dead_code)]
    pub fn get_versions(&self, id: &str) -> anyhow::Result<Vec<VersionRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT version, author, published_at, status, lessons, exercises
             FROM courses WHERE id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![id], |row| {
            Ok(VersionRow {
                version: row.get(0)?,
                author: row.get(1)?,
                published_at: row.get(2)?,
                status: row.get(3)?,
                lessons: row.get(4)?,
                exercises: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Check who owns a course ID (first publisher).
    pub fn get_owner(&self, id: &str) -> anyhow::Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT owner_github FROM courses WHERE id = ?1 ORDER BY created_at ASC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(owner)) => Ok(Some(owner)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn insert_course(&self, c: &NewCourse) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO courses (id, version, name, author, author_github, description,
             language_id, language_display, license, lessons, exercises, has_stages,
             platform, provision, tags, estimated_hours, checksum, published_at,
             min_learnlocal_version, package_filename, created_at,
             owner_github, forked_from_id, forked_from_version, forked_from_author)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25)",
            params![
                c.id,
                c.version,
                c.name,
                c.author,
                c.author_github,
                c.description,
                c.language_id,
                c.language_display,
                c.license,
                c.lessons,
                c.exercises,
                c.has_stages,
                c.platform,
                c.provision,
                c.tags,
                c.estimated_hours,
                c.checksum,
                &now,
                c.min_learnlocal_version,
                c.package_filename,
                &now,
                c.owner_github,
                c.forked_from_id,
                c.forked_from_version,
                c.forked_from_author
            ],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update_course_status(
        &self,
        id: &str,
        version: &str,
        status: &str,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE courses SET status = ?1 WHERE id = ?2 AND version = ?3",
            params![status, id, version],
        )?;
        Ok(())
    }

    pub fn increment_downloads(&self, id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        // Increment downloads on the latest version
        conn.execute(
            "UPDATE courses SET downloads = downloads + 1
             WHERE id = ?1 AND version = (SELECT version FROM courses WHERE id = ?1 ORDER BY created_at DESC LIMIT 1)",
            params![id],
        )?;
        Ok(())
    }

    pub fn upsert_rating(
        &self,
        course_id: &str,
        github_user: &str,
        stars: i32,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO ratings (course_id, github_user, stars, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(course_id, github_user) DO UPDATE SET stars = ?3, created_at = ?4",
            params![course_id, github_user, stars, now],
        )?;
        Ok(())
    }

    pub fn get_ratings(&self, course_id: &str) -> anyhow::Result<RatingsSummary> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COALESCE(AVG(CAST(stars AS REAL)), 0) as avg, COUNT(*) as cnt
             FROM ratings WHERE course_id = ?1",
        )?;
        let result = stmt.query_row(params![course_id], |row| {
            Ok(RatingsSummary {
                average: row.get(0)?,
                count: row.get(1)?,
            })
        })?;
        Ok(result)
    }

    pub fn insert_review(
        &self,
        course_id: &str,
        github_user: &str,
        body: &str,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO reviews (course_id, github_user, body, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![course_id, github_user, body, now],
        )?;
        Ok(())
    }

    pub fn get_reviews(&self, course_id: &str) -> anyhow::Result<Vec<ReviewRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT github_user, body, created_at FROM reviews
             WHERE course_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![course_id], |row| {
            Ok(ReviewRow {
                github_user: row.get(0)?,
                body: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn map_course_row(row: &rusqlite::Row) -> rusqlite::Result<CourseRow> {
        Ok(CourseRow {
            id: row.get("id")?,
            name: row.get("name")?,
            version: row.get("version")?,
            author: row.get("author")?,
            author_github: row.get("author_github")?,
            description: row.get("description")?,
            language_id: row.get("language_id")?,
            language_display: row.get("language_display")?,
            license: row.get("license")?,
            lessons: row.get("lessons")?,
            exercises: row.get("exercises")?,
            has_stages: row.get("has_stages")?,
            platform: row.get("platform")?,
            provision: row.get("provision")?,
            tags: row.get("tags")?,
            estimated_hours: row.get("estimated_hours")?,
            checksum: row.get("checksum")?,
            published_at: row.get("published_at")?,
            min_learnlocal_version: row.get("min_learnlocal_version")?,
            status: row.get("status")?,
            package_filename: row.get("package_filename")?,
            downloads: row.get("downloads")?,
            created_at: row.get("created_at")?,
            owner_github: row.get("owner_github")?,
            forked_from_id: row.get("forked_from_id")?,
            forked_from_version: row.get("forked_from_version")?,
            forked_from_author: row.get("forked_from_author")?,
            avg_rating: row.get("avg_rating")?,
            review_count: row.get("review_count")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.init_schema().unwrap();
        db
    }

    fn sample_course() -> NewCourse {
        NewCourse {
            id: "test-course".to_string(),
            name: "Test Course".to_string(),
            version: "1.0.0".to_string(),
            author: "Tester".to_string(),
            author_github: Some("tester".to_string()),
            description: "A test course.".to_string(),
            language_id: "python3".to_string(),
            language_display: "Python".to_string(),
            license: Some("CC-BY-4.0".to_string()),
            lessons: 3,
            exercises: 10,
            has_stages: false,
            platform: None,
            provision: "system".to_string(),
            tags: r#"["test","python"]"#.to_string(),
            estimated_hours: Some(2.0),
            checksum: "sha256:abc123".to_string(),
            min_learnlocal_version: None,
            package_filename: "test-course-1.0.0.tar.gz".to_string(),
            owner_github: "tester".to_string(),
            forked_from_id: None,
            forked_from_version: None,
            forked_from_author: None,
        }
    }

    #[test]
    fn test_schema_init() {
        let db = test_db();
        let courses = db.list_courses(true).unwrap();
        assert!(courses.is_empty());
    }

    #[test]
    fn test_insert_and_list() {
        let db = test_db();
        db.insert_course(&sample_course()).unwrap();
        let courses = db.list_courses(true).unwrap();
        assert_eq!(courses.len(), 1);
        assert_eq!(courses[0].id, "test-course");
        assert_eq!(courses[0].owner_github, "tester");
    }

    #[test]
    fn test_version_update() {
        let db = test_db();
        db.insert_course(&sample_course()).unwrap();

        // Publish v2.0.0 of same course
        let mut v2 = sample_course();
        v2.version = "2.0.0".to_string();
        v2.lessons = 5;
        v2.exercises = 20;
        v2.package_filename = "test-course-2.0.0.tar.gz".to_string();
        db.insert_course(&v2).unwrap();

        // Listing shows only latest version
        let courses = db.list_courses(true).unwrap();
        assert_eq!(courses.len(), 1);
        assert_eq!(courses[0].version, "2.0.0");
        assert_eq!(courses[0].lessons, 5);

        // Version history shows both
        let versions = db.get_versions("test-course").unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, "2.0.0"); // newest first
        assert_eq!(versions[1].version, "1.0.0");
    }

    #[test]
    fn test_ownership() {
        let db = test_db();
        db.insert_course(&sample_course()).unwrap();

        let owner = db.get_owner("test-course").unwrap();
        assert_eq!(owner, Some("tester".to_string()));

        // Non-existent course has no owner
        let owner = db.get_owner("nonexistent").unwrap();
        assert_eq!(owner, None);
    }

    #[test]
    fn test_fork_lineage() {
        let db = test_db();
        db.insert_course(&sample_course()).unwrap();

        // Fork it
        let fork = NewCourse {
            id: "test-course-extended".to_string(),
            name: "Test Course Extended".to_string(),
            version: "1.0.0".to_string(),
            author: "Forker".to_string(),
            author_github: Some("forker".to_string()),
            description: "An extended version.".to_string(),
            language_id: "python3".to_string(),
            language_display: "Python".to_string(),
            license: Some("CC-BY-4.0".to_string()),
            lessons: 5,
            exercises: 20,
            has_stages: false,
            platform: None,
            provision: "system".to_string(),
            tags: r#"["test","python"]"#.to_string(),
            estimated_hours: Some(3.0),
            checksum: "sha256:def456".to_string(),
            min_learnlocal_version: None,
            package_filename: "test-course-extended-1.0.0.tar.gz".to_string(),
            owner_github: "forker".to_string(),
            forked_from_id: Some("test-course".to_string()),
            forked_from_version: Some("1.0.0".to_string()),
            forked_from_author: Some("Tester".to_string()),
        };
        db.insert_course(&fork).unwrap();

        let c = db.get_course("test-course-extended").unwrap().unwrap();
        assert_eq!(c.forked_from_id.as_deref(), Some("test-course"));
        assert_eq!(c.forked_from_version.as_deref(), Some("1.0.0"));
        assert_eq!(c.forked_from_author.as_deref(), Some("Tester"));
        assert_eq!(c.owner_github, "forker");
    }

    #[test]
    fn test_approved_filter() {
        let db = test_db();
        db.insert_course(&sample_course()).unwrap();
        let courses = db.list_courses(false).unwrap();
        assert!(courses.is_empty());

        db.update_course_status("test-course", "1.0.0", "approved")
            .unwrap();
        let courses = db.list_courses(false).unwrap();
        assert_eq!(courses.len(), 1);
    }

    #[test]
    fn test_increment_downloads() {
        let db = test_db();
        db.insert_course(&sample_course()).unwrap();
        db.increment_downloads("test-course").unwrap();
        db.increment_downloads("test-course").unwrap();
        let c = db.get_course("test-course").unwrap().unwrap();
        assert_eq!(c.downloads, 2);
    }

    #[test]
    fn test_ratings() {
        let db = test_db();
        db.insert_course(&sample_course()).unwrap();
        db.upsert_rating("test-course", "user1", 5).unwrap();
        db.upsert_rating("test-course", "user2", 3).unwrap();

        let summary = db.get_ratings("test-course").unwrap();
        assert_eq!(summary.count, 2);
        assert!((summary.average - 4.0).abs() < 0.01);

        // Upsert updates existing
        db.upsert_rating("test-course", "user1", 1).unwrap();
        let summary = db.get_ratings("test-course").unwrap();
        assert!((summary.average - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_reviews() {
        let db = test_db();
        db.insert_course(&sample_course()).unwrap();
        db.insert_review("test-course", "user1", "Great!").unwrap();
        db.insert_review("test-course", "user2", "Needs work.")
            .unwrap();

        let reviews = db.get_reviews("test-course").unwrap();
        assert_eq!(reviews.len(), 2);
    }

    #[test]
    fn test_one_review_per_user() {
        let db = test_db();
        db.insert_course(&sample_course()).unwrap();
        db.insert_review("test-course", "user1", "First").unwrap();
        assert!(db.insert_review("test-course", "user1", "Second").is_err());
    }

    #[test]
    fn test_duplicate_version_rejected() {
        let db = test_db();
        db.insert_course(&sample_course()).unwrap();
        // Same id + version should fail
        assert!(db.insert_course(&sample_course()).is_err());
    }
}
