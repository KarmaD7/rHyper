mod blk;

trait VirtIODevice {
    fn notify_handler() -> ();
}
