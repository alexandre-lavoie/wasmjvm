use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let projects_path_buf = env::current_dir().unwrap().join("./java/projects");
    let dist_path_buf = env::current_dir().unwrap().join("./java/dist");
    let build_path_buf = env::current_dir().unwrap().join("./java/build");

    if dist_path_buf.exists() {
        fs::remove_dir_all(dist_path_buf.as_path()).unwrap();
    }
    fs::create_dir_all(dist_path_buf.as_path()).unwrap();

    if build_path_buf.exists() {
        fs::remove_dir_all(build_path_buf.as_path()).unwrap();
    }
    fs::create_dir_all(build_path_buf.as_path()).unwrap();

    for project in java_projects(projects_path_buf.as_path()) {
        let project_out_path_buf = build_path_buf.join(project.clone());
        let project_in_path_buf = projects_path_buf.join(project.clone());
        let project_class_path_buf = project_out_path_buf.join("classes");

        if project_out_path_buf.exists() {
            fs::remove_dir_all(project_out_path_buf.as_path()).unwrap();
        }
        fs::create_dir_all(project_out_path_buf.as_path()).unwrap();

        let project_src_path_buf = project_in_path_buf.join("src");
        let mut args = vec![
            "-source".to_string(),
            "1.8".to_string(),
            "-target".to_string(),
            "1.8".to_string(),
            "-extdirs".to_string(),
            project_src_path_buf.to_str().unwrap().to_string(),
            "-sourcepath".to_string(),
            project_src_path_buf.to_str().unwrap().to_string(),
            "-d".to_string(),
            project_class_path_buf.to_str().unwrap().to_string(),
            "-classpath".to_string(),
            project_class_path_buf.to_str().unwrap().to_string(),
        ];
        args.append(&mut java_project_classes(project_in_path_buf.as_path()));
        Command::new("javac").args(args).status().unwrap();

        env::set_current_dir(project_class_path_buf.as_path()).unwrap();

        Command::new("jar")
            .args([
                "cvfe",
                dist_path_buf.join(format!("{}.jar", project)).to_str().unwrap(),
                "Main",
                "./",
            ])
            .status()
            .unwrap();
    }
}

fn java_projects(path: &Path) -> Vec<String> {
    let mut projects: Vec<String> = Vec::new();

    for entry in path.read_dir().unwrap() {
        if let Ok(entry) = entry {
            if entry.file_type().unwrap().is_dir() {
                let project_name = entry.file_name().to_str().unwrap().to_string();
                projects.push(project_name);
            }
        }
    }

    projects
}

fn java_project_classes(path: &Path) -> Vec<String> {
    let mut classes: Vec<String> = Vec::new();

    let mut read_queue = vec![path.to_path_buf()];
    while !read_queue.is_empty() {
        let path_buf = read_queue.pop().unwrap();
        let path = path_buf.as_path();

        for entry in path.read_dir().unwrap() {
            if let Ok(entry) = entry {
                if entry.file_type().unwrap().is_dir() {
                    read_queue.push(entry.path());
                } else if entry.file_name().to_str().unwrap().ends_with(".java") {
                    classes.push(entry.path().to_str().unwrap().to_string());
                }
            }
        }
    }

    classes
}
