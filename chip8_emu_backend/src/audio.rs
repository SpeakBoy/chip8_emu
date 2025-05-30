use rodio::{
    OutputStream, OutputStreamHandle, Sink,
    source::{SineWave, Source},
};

pub struct AudioManager {
    stream_handle: OutputStreamHandle,
    sink: Sink,
    _stream: OutputStream,
}

impl AudioManager {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        Self {
            stream_handle,
            sink,
            _stream: stream,
        }
    }

    pub fn start_beep(&mut self) {
        if self.sink.empty() {
            let beep = SineWave::new(440.0).amplify(0.2).repeat_infinite();
            self.sink.append(beep);
        }
    }

    pub fn stop_beep(&mut self) {
        if !self.sink.empty() {
            self.sink.stop();
            self.sink = Sink::try_new(&self.stream_handle).unwrap();
        }
    }
}
