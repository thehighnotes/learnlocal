use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::course::loader::{find_exercise_file, find_lesson_dir};

// --- Course metadata ---

#[derive(Serialize, Deserialize)]
pub struct CourseMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: Option<String>,
    pub platform: Option<String>,
    pub language: serde_json::Value,
    pub lessons: Vec<LessonRef>,
    pub estimated_minutes_per_lesson: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LessonRef {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub requires: Vec<String>,
}

pub fn read_course_meta(course_path: &Path) -> anyhow::Result<CourseMeta> {
    let yaml = std::fs::read_to_string(course_path.join("course.yaml"))?;
    let meta: CourseMeta = serde_yaml::from_str(&yaml)?;
    Ok(meta)
}

pub fn update_course_meta(
    course_path: &Path,
    name: Option<&str>,
    version: Option<&str>,
    description: Option<&str>,
    author: Option<&str>,
    license: Option<&str>,
) -> anyhow::Result<()> {
    let yaml_path = course_path.join("course.yaml");
    let contents = std::fs::read_to_string(&yaml_path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&contents)?;

    if let serde_yaml::Value::Mapping(ref mut map) = doc {
        if let Some(v) = name {
            map.insert("name".into(), v.into());
        }
        if let Some(v) = version {
            map.insert("version".into(), v.into());
        }
        if let Some(v) = description {
            map.insert("description".into(), v.into());
        }
        if let Some(v) = author {
            map.insert("author".into(), v.into());
        }
        if let Some(v) = license {
            map.insert("license".into(), v.into());
        }
    }

    let output = serde_yaml::to_string(&doc)?;
    std::fs::write(&yaml_path, output)?;
    Ok(())
}

// --- Lessons ---

#[derive(Serialize)]
pub struct LessonInfo {
    pub id: String,
    pub title: String,
    pub dir_name: String,
    pub exercise_count: usize,
    pub exercises: Vec<ExerciseRef>,
}

#[derive(Serialize)]
pub struct ExerciseRef {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub exercise_type: String,
    pub has_stages: bool,
}

pub fn list_lessons(course_path: &Path) -> anyhow::Result<Vec<LessonInfo>> {
    let meta = read_course_meta(course_path)?;
    let lessons_dir = course_path.join("lessons");
    let mut result = Vec::new();

    for lesson_ref in &meta.lessons {
        let lesson_dir = find_lesson_dir(&lessons_dir, &lesson_ref.id)?;
        let dir_name = lesson_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Read lesson.yaml for exercise list
        let lesson_yaml = lesson_dir.join("lesson.yaml");
        let lesson_contents = std::fs::read_to_string(&lesson_yaml)?;
        let lesson: serde_yaml::Value = serde_yaml::from_str(&lesson_contents)?;

        let exercise_ids: Vec<String> = lesson
            .get("exercises")
            .and_then(|e| e.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let exercises_dir = lesson_dir.join("exercises");
        let mut exercises = Vec::new();

        for eid in &exercise_ids {
            if let Ok(ex_path) = find_exercise_file(&exercises_dir, eid) {
                if let Ok(contents) = std::fs::read_to_string(&ex_path) {
                    if let Ok(ex) =
                        serde_yaml::from_str::<crate::course::types::Exercise>(&contents)
                    {
                        exercises.push(ExerciseRef {
                            id: ex.id,
                            title: ex.title,
                            exercise_type: format!("{:?}", ex.exercise_type).to_lowercase(),
                            has_stages: !ex.stages.is_empty(),
                        });
                    }
                }
            }
        }

        result.push(LessonInfo {
            id: lesson_ref.id.clone(),
            title: lesson_ref.title.clone(),
            dir_name,
            exercise_count: exercises.len(),
            exercises,
        });
    }

    Ok(result)
}

pub fn create_lesson(course_path: &Path, id: &str, title: &str) -> anyhow::Result<()> {
    let lessons_dir = course_path.join("lessons");

    // Determine next numeric prefix
    let mut max_num = 0u32;
    if let Ok(entries) = std::fs::read_dir(&lessons_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(prefix) = name.split('-').next() {
                if let Ok(n) = prefix.parse::<u32>() {
                    max_num = max_num.max(n);
                }
            }
        }
    }

    let dir_name = format!("{:02}-{}", max_num + 1, id);
    let lesson_dir = lessons_dir.join(&dir_name);
    std::fs::create_dir_all(&lesson_dir)?;
    std::fs::create_dir_all(lesson_dir.join("exercises"))?;

    // Write lesson.yaml
    let lesson_yaml = format!(
        "id: {}\ntitle: \"{}\"\ncontent: content.md\nexercises: []\n",
        id, title
    );
    std::fs::write(lesson_dir.join("lesson.yaml"), lesson_yaml)?;
    std::fs::write(lesson_dir.join("content.md"), format!("# {}\n", title))?;

    // Update course.yaml to add lesson reference
    let mut meta = read_course_meta(course_path)?;
    meta.lessons.push(LessonRef {
        id: id.to_string(),
        title: title.to_string(),
        requires: vec![],
    });
    write_course_lessons(course_path, &meta.lessons)?;

    Ok(())
}

pub fn delete_lesson(course_path: &Path, id: &str) -> anyhow::Result<()> {
    let lessons_dir = course_path.join("lessons");
    let lesson_dir = find_lesson_dir(&lessons_dir, id)?;

    std::fs::remove_dir_all(&lesson_dir)?;

    // Update course.yaml to remove lesson reference
    let mut meta = read_course_meta(course_path)?;
    meta.lessons.retain(|l| l.id != id);
    write_course_lessons(course_path, &meta.lessons)?;

    Ok(())
}

pub fn reorder_lessons(course_path: &Path, order: &[String]) -> anyhow::Result<()> {
    let lessons_dir = course_path.join("lessons");

    // Rename directories with new numeric prefixes
    for (i, id) in order.iter().enumerate() {
        let current_dir = find_lesson_dir(&lessons_dir, id)?;
        let new_name = format!("{:02}-{}", i + 1, id);
        let new_dir = lessons_dir.join(&new_name);
        if current_dir != new_dir {
            // Rename via temp to avoid collisions
            let temp_dir = lessons_dir.join(format!("__temp__{}", id));
            std::fs::rename(&current_dir, &temp_dir)?;
            std::fs::rename(&temp_dir, &new_dir)?;
        }
    }

    // Update course.yaml lesson order
    let mut meta = read_course_meta(course_path)?;
    let old_lessons = meta.lessons.clone();
    meta.lessons.clear();
    for id in order {
        if let Some(lr) = old_lessons.iter().find(|l| l.id == *id) {
            meta.lessons.push(lr.clone());
        }
    }
    write_course_lessons(course_path, &meta.lessons)?;

    Ok(())
}

fn write_course_lessons(course_path: &Path, lessons: &[LessonRef]) -> anyhow::Result<()> {
    let yaml_path = course_path.join("course.yaml");
    let contents = std::fs::read_to_string(&yaml_path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&contents)?;

    if let serde_yaml::Value::Mapping(ref mut map) = doc {
        let lessons_val = serde_yaml::to_value(lessons)?;
        map.insert("lessons".into(), lessons_val);
    }

    let output = serde_yaml::to_string(&doc)?;
    std::fs::write(&yaml_path, output)?;
    Ok(())
}

// --- Exercises ---

pub fn list_exercises(course_path: &Path, lesson_id: &str) -> anyhow::Result<Vec<ExerciseRef>> {
    let lessons_dir = course_path.join("lessons");
    let lesson_dir = find_lesson_dir(&lessons_dir, lesson_id)?;
    let exercises_dir = lesson_dir.join("exercises");

    // Read lesson.yaml for exercise order
    let lesson_yaml = lesson_dir.join("lesson.yaml");
    let contents = std::fs::read_to_string(&lesson_yaml)?;
    let lesson: serde_yaml::Value = serde_yaml::from_str(&contents)?;

    let exercise_ids: Vec<String> = lesson
        .get("exercises")
        .and_then(|e| e.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let mut result = Vec::new();
    for eid in &exercise_ids {
        if let Ok(ex_path) = find_exercise_file(&exercises_dir, eid) {
            if let Ok(ex_contents) = std::fs::read_to_string(&ex_path) {
                if let Ok(ex) = serde_yaml::from_str::<crate::course::types::Exercise>(&ex_contents)
                {
                    result.push(ExerciseRef {
                        id: ex.id,
                        title: ex.title,
                        exercise_type: format!("{:?}", ex.exercise_type).to_lowercase(),
                        has_stages: !ex.stages.is_empty(),
                    });
                }
            }
        }
    }

    Ok(result)
}

pub fn read_exercise(
    course_path: &Path,
    lesson_id: &str,
    exercise_id: &str,
) -> anyhow::Result<serde_json::Value> {
    let lessons_dir = course_path.join("lessons");
    let lesson_dir = find_lesson_dir(&lessons_dir, lesson_id)?;
    let exercises_dir = lesson_dir.join("exercises");
    let yaml_path = find_exercise_file(&exercises_dir, exercise_id)?;

    let contents = std::fs::read_to_string(&yaml_path)?;
    let exercise: serde_json::Value = serde_yaml::from_str(&contents)?;
    Ok(exercise)
}

pub fn create_exercise(
    course_path: &Path,
    lesson_id: &str,
    exercise_data: &serde_json::Value,
) -> anyhow::Result<()> {
    let id = exercise_data
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Exercise must have an 'id' field"))?;

    let lessons_dir = course_path.join("lessons");
    let lesson_dir = find_lesson_dir(&lessons_dir, lesson_id)?;
    let exercises_dir = lesson_dir.join("exercises");

    // Determine next numeric prefix
    let mut max_num = 0u32;
    if let Ok(entries) = std::fs::read_dir(&exercises_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(prefix) = name.split('-').next() {
                if let Ok(n) = prefix.parse::<u32>() {
                    max_num = max_num.max(n);
                }
            }
        }
    }

    let file_name = format!("{:02}-{}.yaml", max_num + 1, id);

    // Convert JSON to YAML and write
    let yaml_value: serde_yaml::Value = serde_json::from_value(exercise_data.clone())?;
    let yaml_str = serde_yaml::to_string(&yaml_value)?;
    std::fs::write(exercises_dir.join(&file_name), yaml_str)?;

    // Update lesson.yaml exercises list
    add_exercise_to_lesson(&lesson_dir, id)?;

    Ok(())
}

pub fn update_exercise(
    course_path: &Path,
    lesson_id: &str,
    exercise_id: &str,
    exercise_data: &serde_json::Value,
) -> anyhow::Result<()> {
    let lessons_dir = course_path.join("lessons");
    let lesson_dir = find_lesson_dir(&lessons_dir, lesson_id)?;
    let exercises_dir = lesson_dir.join("exercises");
    let yaml_path = find_exercise_file(&exercises_dir, exercise_id)?;

    let yaml_value: serde_yaml::Value = serde_json::from_value(exercise_data.clone())?;
    let yaml_str = serde_yaml::to_string(&yaml_value)?;
    std::fs::write(&yaml_path, yaml_str)?;

    Ok(())
}

pub fn delete_exercise(
    course_path: &Path,
    lesson_id: &str,
    exercise_id: &str,
) -> anyhow::Result<()> {
    let lessons_dir = course_path.join("lessons");
    let lesson_dir = find_lesson_dir(&lessons_dir, lesson_id)?;
    let exercises_dir = lesson_dir.join("exercises");
    let yaml_path = find_exercise_file(&exercises_dir, exercise_id)?;

    std::fs::remove_file(&yaml_path)?;

    // Update lesson.yaml to remove exercise from list
    remove_exercise_from_lesson(&lesson_dir, exercise_id)?;

    Ok(())
}

pub fn reorder_exercises(
    course_path: &Path,
    lesson_id: &str,
    order: &[String],
) -> anyhow::Result<()> {
    let lessons_dir = course_path.join("lessons");
    let lesson_dir = find_lesson_dir(&lessons_dir, lesson_id)?;
    let exercises_dir = lesson_dir.join("exercises");

    // Rename files with new numeric prefixes
    for (i, id) in order.iter().enumerate() {
        let current_path = find_exercise_file(&exercises_dir, id)?;
        let new_name = format!("{:02}-{}.yaml", i + 1, id);
        let new_path = exercises_dir.join(&new_name);
        if current_path != new_path {
            let temp_path = exercises_dir.join(format!("__temp__{}.yaml", id));
            std::fs::rename(&current_path, &temp_path)?;
            std::fs::rename(&temp_path, &new_path)?;
        }
    }

    // Update lesson.yaml exercise order
    update_lesson_exercise_list(&lesson_dir, order)?;

    Ok(())
}

fn add_exercise_to_lesson(lesson_dir: &Path, exercise_id: &str) -> anyhow::Result<()> {
    let yaml_path = lesson_dir.join("lesson.yaml");
    let contents = std::fs::read_to_string(&yaml_path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&contents)?;

    if let serde_yaml::Value::Mapping(ref mut map) = doc {
        let exercises = map
            .entry("exercises".into())
            .or_insert_with(|| serde_yaml::Value::Sequence(vec![]));
        if let serde_yaml::Value::Sequence(ref mut seq) = exercises {
            seq.push(exercise_id.into());
        }
    }

    let output = serde_yaml::to_string(&doc)?;
    std::fs::write(&yaml_path, output)?;
    Ok(())
}

fn remove_exercise_from_lesson(lesson_dir: &Path, exercise_id: &str) -> anyhow::Result<()> {
    let yaml_path = lesson_dir.join("lesson.yaml");
    let contents = std::fs::read_to_string(&yaml_path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&contents)?;

    if let serde_yaml::Value::Mapping(ref mut map) = doc {
        if let Some(serde_yaml::Value::Sequence(ref mut seq)) = map.get_mut("exercises") {
            seq.retain(|v| v.as_str() != Some(exercise_id));
        }
    }

    let output = serde_yaml::to_string(&doc)?;
    std::fs::write(&yaml_path, output)?;
    Ok(())
}

fn update_lesson_exercise_list(lesson_dir: &Path, order: &[String]) -> anyhow::Result<()> {
    let yaml_path = lesson_dir.join("lesson.yaml");
    let contents = std::fs::read_to_string(&yaml_path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&contents)?;

    if let serde_yaml::Value::Mapping(ref mut map) = doc {
        let new_list: Vec<serde_yaml::Value> = order
            .iter()
            .map(|id| serde_yaml::Value::String(id.clone()))
            .collect();
        map.insert("exercises".into(), serde_yaml::Value::Sequence(new_list));
    }

    let output = serde_yaml::to_string(&doc)?;
    std::fs::write(&yaml_path, output)?;
    Ok(())
}
