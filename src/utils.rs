use rand::seq::SliceRandom;

pub fn get_random_comments(comments: &[String], count: usize) -> Vec<String> {
    if comments.is_empty() {
        return vec![];
    }
    
    let mut rng = rand::thread_rng();
    let selected_count = std::cmp::min(count, comments.len());
    
    comments
        .choose_multiple(&mut rng, selected_count)
        .cloned()
        .collect()
}
