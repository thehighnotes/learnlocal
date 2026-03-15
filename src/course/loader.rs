use super::types::*;
use crate::error::{LearnLocalError, Result};
use std::path::Path;

/// Load only course.yaml metadata — no lessons or exercises.
pub fn load_course_info(path: &Path) -> Result<CourseInfo> {
    let course_yaml = path.join("course.yaml");
    if !course_yaml.exists() {
        return Err(LearnLocalError::CourseLoad(format!(
            "course.yaml not found in {}",
            path.display()
        )));
    }

    let contents = std::fs::read_to_string(&course_yaml)?;
    let course: Course = serde_yaml::from_str(&contents)?;

    let dir_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let step_commands = crate::exec::toolcheck::extract_step_commands(&course.language);

    // Count exercises and collect env commands by scanning lesson directories
    let total_exercise_count = count_total_exercises(path, &course);
    let env_commands = collect_env_commands(path, &course);

    let provision = course.language.provision.clone();

    Ok(CourseInfo {
        dir_name,
        lesson_ids: course.lessons.iter().map(|l| l.id.clone()).collect(),
        lesson_titles: course.lessons.iter().map(|l| l.title.clone()).collect(),
        lesson_count: course.lessons.len(),
        language_name: course.language.display_name,
        license: course.license,
        platform: course.platform,
        estimated_minutes_per_lesson: course.estimated_minutes_per_lesson,
        name: course.name,
        version: course.version,
        description: course.description,
        author: course.author,
        source_dir: path.to_path_buf(),
        step_commands,
        env_commands,
        total_exercise_count,
        provision,
    })
}

/// Count total exercises without fully loading them — reads lesson.yaml exercise lists.
fn count_total_exercises(path: &Path, course: &Course) -> Option<usize> {
    let lessons_dir = path.join("lessons");
    if !lessons_dir.exists() {
        return None;
    }

    let mut total = 0usize;
    for lesson_ref in &course.lessons {
        if let Ok(lesson_dir) = find_lesson_dir(&lessons_dir, &lesson_ref.id) {
            let lesson_yaml = lesson_dir.join("lesson.yaml");
            if let Ok(contents) = std::fs::read_to_string(&lesson_yaml) {
                if let Ok(lesson) = serde_yaml::from_str::<Lesson>(&contents) {
                    total += lesson.exercises.len();
                }
            }
        }
    }

    Some(total)
}

/// Collect unique environment commands from exercise YAML files.
/// Scans each exercise for environment.setup/services/teardown commands.
fn collect_env_commands(path: &Path, course: &Course) -> Vec<String> {
    let lessons_dir = path.join("lessons");
    if !lessons_dir.exists() {
        return Vec::new();
    }

    let mut seen = std::collections::HashSet::new();
    let mut cmds = Vec::new();

    for lesson_ref in &course.lessons {
        if let Ok(lesson_dir) = find_lesson_dir(&lessons_dir, &lesson_ref.id) {
            let lesson_yaml = lesson_dir.join("lesson.yaml");
            if let Ok(contents) = std::fs::read_to_string(&lesson_yaml) {
                if let Ok(lesson) = serde_yaml::from_str::<Lesson>(&contents) {
                    let exercises_dir = lesson_dir.join("exercises");
                    for ex_id in &lesson.exercises {
                        // Try directory format first, then flat file format
                        let ex_yaml = exercises_dir.join(ex_id).join("exercise.yaml");
                        let ex_contents = std::fs::read_to_string(&ex_yaml).or_else(|_| {
                            // Try flat file: exercises/NN-id.yaml or exercises/id.yaml
                            find_exercise_file(&exercises_dir, ex_id)
                                .ok()
                                .and_then(|p| std::fs::read_to_string(p).ok())
                                .ok_or(std::io::Error::new(
                                    std::io::ErrorKind::NotFound,
                                    "not found",
                                ))
                        });
                        if let Ok(ex_contents) = ex_contents {
                            if let Ok(exercise) = serde_yaml::from_str::<Exercise>(&ex_contents) {
                                if let Some(ref env) = exercise.environment {
                                    for c in crate::exec::toolcheck::extract_env_commands(env) {
                                        if seen.insert(c.clone()) {
                                            cmds.push(c);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    cmds
}

pub fn load_course(path: &Path) -> Result<Course> {
    let course_yaml = path.join("course.yaml");
    if !course_yaml.exists() {
        return Err(LearnLocalError::CourseLoad(format!(
            "course.yaml not found in {}",
            path.display()
        )));
    }

    let contents = std::fs::read_to_string(&course_yaml)?;
    let mut course: Course = serde_yaml::from_str(&contents)?;
    course.source_dir = path.to_path_buf();

    let lessons_dir = path.join("lessons");
    if !lessons_dir.exists() {
        return Err(LearnLocalError::CourseLoad(format!(
            "lessons/ directory not found in {}",
            path.display()
        )));
    }

    for lesson_ref in &course.lessons {
        let lesson = load_lesson(&lessons_dir, &lesson_ref.id, &course.language.extension)?;
        course.loaded_lessons.push(lesson);
    }

    Ok(course)
}

fn load_lesson(lessons_dir: &Path, lesson_id: &str, extension: &str) -> Result<Lesson> {
    // Find the lesson directory — it may be prefixed with a number like "01-variables"
    let lesson_dir = find_lesson_dir(lessons_dir, lesson_id)?;

    let lesson_yaml = lesson_dir.join("lesson.yaml");
    if !lesson_yaml.exists() {
        return Err(LearnLocalError::CourseLoad(format!(
            "lesson.yaml not found in {}",
            lesson_dir.display()
        )));
    }

    let contents = std::fs::read_to_string(&lesson_yaml)?;
    let mut lesson: Lesson = serde_yaml::from_str(&contents)?;

    // Load content.md
    let content_path = lesson_dir.join(&lesson.content);
    if content_path.exists() {
        lesson.content_markdown = std::fs::read_to_string(&content_path)?;
        lesson.content_sections = split_content_sections(&lesson.content_markdown);
    }

    // Load exercises
    let exercises_dir = lesson_dir.join("exercises");
    if !exercises_dir.exists() {
        return Err(LearnLocalError::CourseLoad(format!(
            "exercises/ directory not found in {}",
            lesson_dir.display()
        )));
    }

    for exercise_id in &lesson.exercises {
        let exercise = load_exercise(&exercises_dir, exercise_id, extension)?;
        lesson.loaded_exercises.push(exercise);
    }

    Ok(lesson)
}

fn find_lesson_dir(lessons_dir: &Path, lesson_id: &str) -> Result<std::path::PathBuf> {
    // Try exact match first
    let exact = lessons_dir.join(lesson_id);
    if exact.exists() {
        return Ok(exact);
    }

    // Try numbered prefix (e.g., "01-variables" for id "variables")
    if let Ok(entries) = std::fs::read_dir(lessons_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Strip leading numeric prefix segments (e.g., "01-" or "01-02-")
            // and check if the remainder matches the lesson ID
            if let Some((prefix, suffix)) = name.split_once('-') {
                if prefix.chars().all(|c| c.is_ascii_digit()) && suffix == lesson_id {
                    return Ok(entry.path());
                }
            }
        }
    }

    Err(LearnLocalError::CourseLoad(format!(
        "Lesson directory for '{}' not found in {}",
        lesson_id,
        lessons_dir.display()
    )))
}

fn load_exercise(exercises_dir: &Path, exercise_id: &str, _extension: &str) -> Result<Exercise> {
    // Try exact match first, then with .yaml extension
    let yaml_path = find_exercise_file(exercises_dir, exercise_id)?;

    let contents = std::fs::read_to_string(&yaml_path)?;
    let exercise: Exercise = serde_yaml::from_str(&contents)?;

    Ok(exercise)
}

/// Split markdown content by H2 (`## `) headers into sections.
/// The text before the first H2 is discarded (it's the lesson intro shown in LessonContent).
/// Each section includes the H2 header line and all text until the next H2.
fn split_content_sections(markdown: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut current_section = String::new();
    let mut in_section = false;

    for line in markdown.lines() {
        if line.starts_with("## ") {
            if in_section && !current_section.trim().is_empty() {
                sections.push(current_section.trim().to_string());
            }
            current_section = String::new();
            current_section.push_str(line);
            current_section.push('\n');
            in_section = true;
        } else if in_section {
            current_section.push_str(line);
            current_section.push('\n');
        }
    }

    if in_section && !current_section.trim().is_empty() {
        sections.push(current_section.trim().to_string());
    }

    sections
}

/// Split markdown content by H2 (`## `) headers into display sections,
/// keeping the intro (text before first H2) as section 0.
/// Returns: [intro, h2_section_1, h2_section_2, ...]
/// If no H2 headers: returns [full_content] (single element).
/// If content starts with H2 (no intro): first section is the first H2 block.
pub fn split_display_sections(markdown: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut current_section = String::new();
    let mut found_h2 = false;

    for line in markdown.lines() {
        if line.starts_with("## ") {
            // Push whatever we've accumulated (intro or previous H2 section)
            let trimmed = current_section.trim().to_string();
            if !trimmed.is_empty() {
                sections.push(trimmed);
            }
            current_section = String::new();
            current_section.push_str(line);
            current_section.push('\n');
            found_h2 = true;
        } else {
            current_section.push_str(line);
            current_section.push('\n');
        }
    }

    // Push the last section
    let trimmed = current_section.trim().to_string();
    if !trimmed.is_empty() {
        sections.push(trimmed);
    }

    // If no H2 was found and we have content, sections already has [full_content]
    // If no content at all, return empty vec
    let _ = found_h2; // suppress unused warning
    sections
}

fn find_exercise_file(exercises_dir: &Path, exercise_id: &str) -> Result<std::path::PathBuf> {
    // Try "id.yaml"
    let direct = exercises_dir.join(format!("{}.yaml", exercise_id));
    if direct.exists() {
        return Ok(direct);
    }

    // Try numbered prefix "NN-id.yaml"
    if let Ok(entries) = std::fs::read_dir(exercises_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(stem) = name.strip_suffix(".yaml") {
                if let Some(suffix) = stem.split_once('-').map(|(_, s)| s) {
                    if suffix == exercise_id {
                        return Ok(entry.path());
                    }
                }
            }
        }
    }

    Err(LearnLocalError::CourseLoad(format!(
        "Exercise file for '{}' not found in {}",
        exercise_id,
        exercises_dir.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal-course")
    }

    #[test]
    fn test_load_course_missing_dir() {
        let result = load_course(Path::new("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_split_content_sections_basic() {
        let md = "# Lesson Title\nIntro text\n\n## Section One\nContent one.\n\n## Section Two\nContent two.\n";
        let sections = super::split_content_sections(md);
        assert_eq!(sections.len(), 2);
        assert!(sections[0].starts_with("## Section One"));
        assert!(sections[0].contains("Content one."));
        assert!(sections[1].starts_with("## Section Two"));
        assert!(sections[1].contains("Content two."));
    }

    #[test]
    fn test_split_content_sections_no_h2() {
        let md = "# Just a heading\nSome text\n";
        let sections = super::split_content_sections(md);
        assert_eq!(sections.len(), 0);
    }

    #[test]
    fn test_split_content_sections_single() {
        let md = "## Only One\nContent here.\n";
        let sections = super::split_content_sections(md);
        assert_eq!(sections.len(), 1);
        assert!(sections[0].starts_with("## Only One"));
    }

    #[test]
    fn test_split_display_sections_with_intro() {
        let md = "# Lesson Title\nIntro text here.\n\n## Section One\nContent one.\n\n## Section Two\nContent two.\n";
        let sections = split_display_sections(md);
        assert_eq!(sections.len(), 3);
        assert!(sections[0].starts_with("# Lesson Title"));
        assert!(sections[0].contains("Intro text"));
        assert!(sections[1].starts_with("## Section One"));
        assert!(sections[2].starts_with("## Section Two"));
    }

    #[test]
    fn test_split_display_sections_no_h2() {
        let md = "# Just a heading\nSome text\nMore text\n";
        let sections = split_display_sections(md);
        assert_eq!(sections.len(), 1);
        assert!(sections[0].contains("Just a heading"));
        assert!(sections[0].contains("Some text"));
    }

    #[test]
    fn test_split_display_sections_starts_with_h2() {
        let md = "## First Section\nContent one.\n\n## Second Section\nContent two.\n";
        let sections = split_display_sections(md);
        assert_eq!(sections.len(), 2);
        assert!(sections[0].starts_with("## First Section"));
        assert!(sections[1].starts_with("## Second Section"));
    }

    #[test]
    fn test_load_course_fixture() {
        let dir = fixtures_dir();
        if !dir.exists() {
            return; // Skip if fixtures not yet created
        }
        let course = load_course(&dir).unwrap();
        assert_eq!(course.name, "Test Course");
        assert!(!course.loaded_lessons.is_empty());
    }
}
