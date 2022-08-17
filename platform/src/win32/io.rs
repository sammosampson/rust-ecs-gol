pub fn build_target_file_path_name(file_name: &str) -> String {
    format!("target\\{}", file_name)
}

pub fn build_data_file_name(file_name: &str) -> String {
    format!("..\\..\\data\\{}", file_name)
}