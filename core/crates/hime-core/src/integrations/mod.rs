pub mod drive;
pub mod github;

pub use drive::{DriveError, DriveFileMetadata, GoogleDriveAdapter};
pub use github::{GitHubAdapter, GitHubError, IssueInfo, PullRequestInfo, RepositoryInfo};

