use std::io::Write;
use std::sync::mpsc;

pub struct ChannelWriter{
    pub sender: mpsc::Sender<String>,
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = self.sender.send(String::from_utf8_lossy(buf).into_owned());
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        // this function is needed to implement the Write trait so just return Ok(())
        Ok(())
    }
}

