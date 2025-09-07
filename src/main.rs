use jack::{Client, ClientOptions, Control, ProcessHandler, ProcessScope, Port, AudioOut};
use std::f32::consts::PI;

struct Sine {
    phase: f32,
    step: f32,
}
impl Sine {
    fn new(freq_hz: f32, sample_rate: f32) -> Self {
        Self { phase: 0.0, step: (2.0 * PI * freq_hz) / sample_rate }
    }
    #[inline]
    fn next(&mut self) -> f32 {
        let v = self.phase.sin();
        self.phase += self.step;
        if self.phase >= 2.0 * PI { self.phase -= 2.0 * PI; }
        v
    }
}

struct Handler {
    left: Port<AudioOut>,
    right: Port<AudioOut>,
    osc: Sine,
}

impl ProcessHandler for Handler {
    fn process(&mut self, _client: &Client, ps: &ProcessScope) -> Control {
        let out_l = self.left.as_mut_slice(ps);
        let out_r = self.right.as_mut_slice(ps);
        for (l, r) in out_l.iter_mut().zip(out_r.iter_mut()) {
            let s = self.osc.next() * 0.2;
            *l = s;
            *r = s;
        }
        Control::Continue
    }
}

fn main() {
    // Open a JACK client but don't try to start a standalone server (we're on PipeWire).
    let (client, _status) =
        Client::new("jack_sine", ClientOptions::NO_START_SERVER).expect("JACK not available");

    let sr = client.sample_rate() as f32;
    eprintln!("JACK sample rate: {}", sr);

    let out_l = client.register_port("out_l", AudioOut::default()).expect("port L");
    let out_r = client.register_port("out_r", AudioOut::default()).expect("port R");

    let handler = Handler { left: out_l, right: out_r, osc: Sine::new(440.0, sr) };

    // Start audio processing
    let _active = client.activate_async((), handler).expect("activate failed");

    eprintln!("Running. Connect to playback with qpwgraph or:");
    eprintln!(r#"  jack_connect "jack_sine:out_l" "USB Audio Analog Stereo:playback_FL""#);
    eprintln!(r#"  jack_connect "jack_sine:out_r" "USB Audio Analog Stereo:playback_FR""#);
    eprintln!("Ctrl+C to quit.");

    // park the main thread
    loop { std::thread::park(); }
}
