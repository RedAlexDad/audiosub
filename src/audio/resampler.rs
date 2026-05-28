use anyhow::{Context, Result};
use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};

pub struct AudioResampler {
    resampler: Fft<f32>,
    input_rate: u32,
    output_rate: u32,
    input_needed: usize,
    output_needed: usize,
    buffer: Vec<f32>,
}

impl AudioResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Result<Self> {
        let chunk_size = 2048;
        let resampler = Fft::<f32>::new(
            input_rate as usize,
            output_rate as usize,
            chunk_size,
            2,
            1,
            FixedSync::Both,
        )
        .context("Failed to create FFT resampler")?;

        let input_needed = resampler.input_frames_next();
        let output_needed = resampler.output_frames_next();

        Ok(Self {
            resampler,
            input_rate,
            output_rate,
            input_needed,
            output_needed,
            buffer: Vec::new(),
        })
    }

    pub fn process(&mut self, input: &[f32]) -> Result<Vec<f32>> {
        self.buffer.extend_from_slice(input);
        let mut output = Vec::new();

        let mut consumed = 0;
        while self.buffer.len() - consumed >= self.input_needed {
            let chunk = &self.buffer[consumed..consumed + self.input_needed];

            let input_adapter =
                InterleavedSlice::new(chunk, 1, self.input_needed).context("Failed to create input adapter")?;
            let mut out_scratch = vec![0.0f32; self.output_needed];
            let mut output_adapter = InterleavedSlice::new_mut(&mut out_scratch, 1, self.output_needed)
                .context("Failed to create output adapter")?;

            let (_, nbr_out) = self
                .resampler
                .process_into_buffer(&input_adapter, &mut output_adapter, None)
                .context("Resampling failed")?;

            output.extend_from_slice(&out_scratch[..nbr_out]);
            consumed += self.input_needed;
        }

        self.buffer.drain(..consumed);
        Ok(output)
    }

    pub fn flush(&mut self) -> Result<Vec<f32>> {
        if self.buffer.is_empty() {
            return Ok(Vec::new());
        }
        let pad = self.input_needed - self.buffer.len();
        self.buffer.extend(std::iter::repeat_n(0.0, pad));

        let input_adapter =
            InterleavedSlice::new(&self.buffer, 1, self.input_needed).context("Failed to create input adapter")?;
        let mut out_scratch = vec![0.0f32; self.output_needed];
        let mut output_adapter = InterleavedSlice::new_mut(&mut out_scratch, 1, self.output_needed)
            .context("Failed to create output adapter")?;

        self.resampler
            .process_into_buffer(&input_adapter, &mut output_adapter, None)
            .context("Resampling flush failed")?;

        self.buffer.clear();
        Ok(out_scratch)
    }

    pub fn input_rate(&self) -> u32 {
        self.input_rate
    }

    pub fn output_rate(&self) -> u32 {
        self.output_rate
    }

    pub fn input_needed(&self) -> usize {
        self.input_needed
    }

    pub fn reset(&mut self) {
        self.resampler.reset();
        self.buffer.clear();
        self.input_needed = self.resampler.input_frames_next();
        self.output_needed = self.resampler.output_frames_next();
    }
}
