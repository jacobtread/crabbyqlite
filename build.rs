fn main() {
    #[cfg(windows)]
    {
        use winres::WindowsResource;
        let mut res = WindowsResource::new();
        res.set_icon("assets/logo.ico");
        res.compile().unwrap();
    }
}
