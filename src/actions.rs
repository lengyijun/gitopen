use crate::match_logic::{
    get_commit_link, get_line_number_link, parse_path_and_line_arg, parse_url_from_git,
};
use anyhow::anyhow;
use anyhow::Result as AnyhowResult;
use regex::Regex;
use std::io::{Error, ErrorKind};
use std::process::{Command, Stdio};

fn get_local_branch_name() -> AnyhowResult<String> {
    let git_repo = Command::new("git")
        .args(["symbolic-ref", "HEAD"])
        .stdout(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8(git_repo.stdout)?;
    if stdout.starts_with("refs/heads/") {
        Ok(stdout[11..].trim().to_string())
    } else {
        Err(Error::new(ErrorKind::Other, "oh no!").into())
    }
}

fn get_remote_branch_name(local_branch_name: String) -> AnyhowResult<String> {
    let git_repo = Command::new("git")
        .args([
            "config",
            "--get",
            &format!("branch.{}.remote", local_branch_name),
        ])
        .stdout(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8(git_repo.stdout)?.trim().to_string();
    Ok(stdout)
}

// TODO: Add caching (`cached` crate)
fn get_parsed_url(remote_branch_name: String) -> AnyhowResult<String> {
    let git_repo = Command::new("git")
        .args([
            "config",
            "--get",
            &format!("remote.{}.url", remote_branch_name),
        ])
        .stdout(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8(git_repo.stdout)?;
    let parsed_url = parse_url_from_git(&stdout)?;

    Ok(parsed_url)
}

pub fn open_repo() -> AnyhowResult<()> {
    let local_branch_name = get_local_branch_name()?;
    let remote_branch_name = get_remote_branch_name(local_branch_name)?;
    let parsed_url = get_parsed_url(remote_branch_name)?;
    webbrowser::open(&parsed_url)?;
    Ok(())
}

pub fn open_commit(commit_sha: &str) -> AnyhowResult<()> {
    let local_branch_name = get_local_branch_name()?;
    let remote_branch_name = get_remote_branch_name(local_branch_name)?;
    let parsed_url = get_parsed_url(remote_branch_name)?;
    let commit_link = get_commit_link(parsed_url, commit_sha);

    webbrowser::open(&commit_link)?;
    Ok(())
}

pub fn open_at_line_number(input: &str) -> AnyhowResult<()> {
    let file_at_line = parse_path_and_line_arg(input, ':')?;
    let local_branch_name = get_local_branch_name()?;
    let remote_branch_name = get_remote_branch_name(local_branch_name)?;
    let parsed_url = get_parsed_url(remote_branch_name)?;
    let line_number_link =
        get_line_number_link(&parsed_url, file_at_line.filepath, file_at_line.line_number)?;

    webbrowser::open(&line_number_link)?;
    Ok(())
}

pub fn push_and_open_pr() -> AnyhowResult<()> {
    let current_branch = Command::new("git")
        .args(["branch", "--show-current"])
        .stdout(Stdio::piped())
        .output()?;
    let current_branch_text = &String::from_utf8(current_branch.stdout)?;
    let current_branch_text_stripped = current_branch_text.trim();
    let output_from_push = Command::new("git")
        .args(["push", "origin", current_branch_text_stripped])
        .stderr(Stdio::piped())
        .output()?;
    let pr_re = Regex::new(r"remote:.*(https\S*)\s*\n")?;
    let output_from_push_text = String::from_utf8(output_from_push.stderr)?;
    let captured = pr_re
        .captures(&output_from_push_text)
        .ok_or_else(|| anyhow!("Error capturing PR url"))?;
    webbrowser::open(&captured[1])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_pr_parsing_from_output() {
        let output = r#"Counting objects: 4, done.
Delta compression using up to 12 threads.
Compressing objects: 100% (4/4), done.
Writing objects: 100% (4/4), 3.01 KiB | 3.01 MiB/s, done.
Total 4 (delta 2), reused 0 (delta 0)
remote: Resolving deltas: 100% (2/2), completed with 2 local objects.
remote:
remote: Create a pull request for 'feat/add-more-pokemons' on GitHub by visiting:
remote:      https://github.com/tobiasbueschel/awesome-pokemon/pull/new/feat/add-more-pokemons
remote:
To github.com:tobiasbueschel/awesome-pokemon.git
 * [new branch]      feat/add-more-pokemons -> feat/add-more-pokemons"#;
        let re = Regex::new(r"remote:.*(https\S*)\s*\n").unwrap();
        let captured = re.captures(output).unwrap();
        println!("{:?}", &captured[1]);
        assert!(&captured[1].starts_with("https"));
        assert!(&captured[1].ends_with("add-more-pokemons"));
    }
}
