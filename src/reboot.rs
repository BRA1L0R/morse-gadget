use esp32c3_hal::reset::software_reset;

pub unsafe fn reboot_download() -> ! {
    let peripherals = esp32c3::Peripherals::steal();

    peripherals
        .RTC_CNTL
        .option1()
        .write(|reg| reg.force_download_boot().set_bit());

    log::info!("Rebooting into download mode!");

    software_reset();
    unreachable!()
}
