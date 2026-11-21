fn main() {
    let exit_code = ebook_ui::run_app();
    std::process::exit(exit_code.value());
}
