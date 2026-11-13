fn main() {
    let job = timed::job();
    println!("{}", job.report());
    println!("{}", job.queue.len());
    println!("{:?}", timed::wait());
    println!("{:?} {:?}", timed::first_over(&job.queue, 1), timed::first_over(&job.queue, 5));
}
