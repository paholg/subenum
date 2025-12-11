use subenum::subenum;

trait Processor {
    type Input;
    type Output;
}

struct Audio;
impl Processor for Audio {
    type Input = Vec<u8>;
    type Output = Vec<f32>;
}

struct Video;
impl Processor for Video {
    type Input = Vec<u32>;
    type Output = Vec<u8>;
}

// Complex Usage
#[subenum(InputOnly, OutputOnly)]
enum Pipeline<P>
where
    P: Processor,
    P::Input: PartialEq + std::fmt::Debug,
    P::Output: Clone,
{
    #[subenum(InputOnly)]
    Source(P::Input),

    #[subenum(OutputOnly)]
    Sink(P::Output),
}

#[test]
fn test_associated_type_usage() {
    // 1. InputOnly
    // The variant field is `P::Input`.
    // Does the macro realize that `P` is required?
    // Since `P::Input` is a projection of `P`, `P` MUST be retained.

    let src_audio: InputOnly<Audio> = InputOnly::Source(vec![1, 2, 3]);
    let src_video: InputOnly<Video> = InputOnly::Source(vec![10, 20]);

    match src_audio {
        InputOnly::Source(bytes) => assert_eq!(bytes.len(), 3),
    }
    match src_video {
        InputOnly::Source(bytes) => assert_eq!(bytes.len(), 2),
    }

    // 2. OutputOnly
    // Similarly, checks P::Output usage
    let sink: OutputOnly<Audio> = OutputOnly::Sink(vec![0.0, 1.0]);
    match sink {
        OutputOnly::Sink(floats) => assert_eq!(floats.len(), 2),
    }
}
