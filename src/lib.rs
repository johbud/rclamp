#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod helpers;
mod projects;
mod tasks;
mod workfiles;
pub use app::Rclamp;
pub use projects::Project;
pub use tasks::TaskTreeNode;
pub use workfiles::File;

#[cfg(test)]
mod tests {

    use crate::helpers::sanitize_string;

    #[test]
    fn test_sanitizer() {
        assert_eq!(
            sanitize_string(String::from("ABC/?<-ÅÄÖ_xyz_1234-åäö%^<??<>//")),
            String::from("abc_aao_xyz_1234_aao")
        );
    }
}
