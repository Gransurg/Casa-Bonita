pub const PARAGRAPHS_PER_PAGE: usize = 40;
const WORDS_PER_MINUTE: usize = 190;

pub fn paginate_chapter_content(content: &str) -> Vec<Vec<String>> {
    let paragraphs: Vec<String> = content
        .split('\n')
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();

    if paragraphs.is_empty() {
        return vec![Vec::new()];
    }

    paragraphs
        .chunks(PARAGRAPHS_PER_PAGE)
        .map(|chunk| chunk.to_vec())
        .collect()
}

pub fn estimate_remaining_minutes(pages: &[Vec<String>], current_page_idx: usize) -> usize {
    if current_page_idx >= pages.len() {
        return 0;
    }

    let remaining_words: usize = pages[current_page_idx..]
        .iter()
        .flat_map(|page| page.iter())
        .map(|para| para.split_whitespace().count())
        .sum();

    if remaining_words == 0 {
        return 0;
    }

    remaining_words.div_ceil(WORDS_PER_MINUTE).max(1)
}
