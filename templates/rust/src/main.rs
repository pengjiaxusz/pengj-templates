fn main() {
    println!("{% if options["chinese_programming"] and options["skill_lang"] | default("zh") != "en" %}你好，{{ project_name }}！{% else %}Hello, {{ project_name }}!{% endif %}");
}
