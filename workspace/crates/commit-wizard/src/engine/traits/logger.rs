pub trait LoggerTrait: Send + Sync {
    fn heading(&self, msg: &str);
    fn debug(&self, msg: &str);
    fn trace(&self, msg: &str);
    fn error_kv(&self, key: &str, value: &str);
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
    fn kv(&self, key: &str, value: &str);
}
