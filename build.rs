fn main() {
    // Compile resource files during build
    glib_build_tools::compile_resources(
        &["src/ui"],
        "src/ui/resources.gresource.xml",
        "resources.gresource",
    );
}
