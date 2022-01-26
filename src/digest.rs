use data_encoding::BASE32;
use flate2::read::GzDecoder;
use sha1::{Digest, Sha1};
use std::io::BufWriter;
use std::io::Read;

pub fn compute_digest<R: Read>(input: &mut R) -> std::io::Result<String> {
    let sha1 = Sha1::new();

    let mut buffered = BufWriter::new(sha1);
    std::io::copy(input, &mut buffered)?;

    let result = buffered.into_inner()?.finalize();

    let mut output = String::new();
    BASE32.encode_append(&result, &mut output);

    Ok(output)
}

pub fn compute_digest_gz<R: Read>(input: &mut R) -> std::io::Result<String> {
    compute_digest(&mut GzDecoder::new(input))
}

pub struct Computer {
    writer: BufWriter<Sha1>,
}

impl Computer {
    pub fn digest<R: Read>(&mut self, input: &mut R) -> std::io::Result<String> {
        std::io::copy(input, &mut self.writer)?;

        let result = self.writer.get_mut().finalize_reset();

        Ok(BASE32.encode(&result))
    }
}

impl Default for Computer {
    fn default() -> Computer {
        Computer {
            writer: BufWriter::new(Sha1::new()),
        }
    }
}
