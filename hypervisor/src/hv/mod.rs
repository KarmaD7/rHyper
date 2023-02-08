pub fn run() {

}

#[naked]
unsafe extern "C" fn test_guest() -> ! {
  core::arch::asm!(
    "wfi",
    options(noreturn)
  )
}


