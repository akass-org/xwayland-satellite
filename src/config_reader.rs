use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::SystemTime;
use std::{env, fs};

pub struct Config {
    path: String,
    scale: f32,
    mtime: Option<SystemTime>,
}

impl Config {
    pub fn new() -> Self {
        let home = env::var("HOME").expect("HOME env not set");
        let path = {
            let dms = format!("{}/.config/niri/dms/outputs.kdl", home);
            let default = format!("{}/.config/niri/config.kdl", home);

            if Path::new(&dms).exists() {
                dms
            } else {
                default
            }
        };
        let scale = 1.0;
        Self {
            path,
            scale,
            mtime: None,
        }
    }

    pub fn get_scale(&mut self) -> f32 {
        self.check_update();
        self.scale
    }

    fn check_update(&mut self) {
        let mtime = get_file_mtime(&self.path);
        // println!("Checking config file: {}, mtime: {:?}", self.path, mtime);
        if mtime.is_some() && mtime != self.mtime {
            println!("File changed, reloading...");
            match self.get_scale_from_file() {
                Ok(scales) => {
                    let max_scale = scales
                        .values()
                        .copied() // 复制 f32 值，方便返回
                        .max_by(|a, b| a.partial_cmp(b).unwrap()); // unwrap 假设没有 NaN
                    self.scale = max_scale.unwrap();
                }
                Err(e) => {
                    println!("Error: {}", e);
                    self.scale = 1.0;
                }
            }
            self.mtime = mtime;
        }
    }

    fn get_scale_from_file(&mut self) -> io::Result<HashMap<String, f32>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        // 正则表达式匹配 output 块和 scale 行
        let output_regex = Regex::new(r#"^\s*output\s+"([^"]+)""#).unwrap(); // 匹配output块
        let scale_regex = Regex::new(r"^\s*scale\s+([0-9\.]+)\s*$").unwrap(); // 匹配 scale 行

        let mut scales: HashMap<String, f32> = HashMap::new();
        let mut current_output = String::new();

        // 逐行读取文件
        for line in reader.lines() {
            let line = line?;

            // 查找 output 块
            if let Some(captures) = output_regex.captures(&line) {
                current_output = captures[1].to_string();
            }

            // 查找 scale 行并提取
            if let Some(captures) = scale_regex.captures(&line) {
                if !current_output.is_empty() {
                    if let Ok(scale) = captures[1].parse::<f32>() {
                        scales.insert(current_output.clone(), scale);
                    }
                }
            }
        }

        println!("Scales found: {:?}", scales);

        if scales.is_empty() {
            Err(io::Error::new(io::ErrorKind::NotFound, "No scales found"))
        } else {
            Ok(scales)
        }
    }
}

fn get_file_mtime(file_path: &str) -> Option<SystemTime> {
    let metadata = fs::metadata(file_path).ok()?;
    metadata.modified().ok()
}
