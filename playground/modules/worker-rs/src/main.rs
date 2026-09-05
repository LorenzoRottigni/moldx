use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Job {
    id: u32,
    payload: String,
    status: String,
}

fn process_job(mut job: Job) -> Job {
    job.status = "processed".to_string();
    job
}

fn main() {
    let mut jobs: Vec<Job> = (1..=5)
        .map(|id| Job {
            id,
            payload: format!("job-{id}"),
            status: String::new(),
        })
        .collect();

    println!(
        "[worker-rs] starting with {} jobs",
        jobs.len()
    );

    for job in &mut jobs {
        *job = process_job(job.clone());
    }

    let json = serde_json::to_string_pretty(&jobs).unwrap();
    println!("{json}");
    println!("[worker-rs] processed {} jobs", jobs.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_job_marks_processed() {
        let job = Job {
            id: 1,
            payload: "x".into(),
            status: String::new(),
        };
        assert_eq!(process_job(job).status, "processed");
    }
}
