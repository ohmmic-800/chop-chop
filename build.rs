fn main() {
    // Compile resource files during build
    glib_build_tools::compile_resources(&["data/icons", "src/ui"], "gresource.xml", "gresource");
}
