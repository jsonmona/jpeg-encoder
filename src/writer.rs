use crate::marker::Marker;

use std::io::{Write, Result as IOResult};
use crate::huffman::{HuffmanTable, CodingClass};
use crate::quantization::QuantizationTable;

#[derive(Debug)]
pub enum Density {
    None,
    Inch { x: u16, y: u16 },
    Centimeter { x: u16, y: u16 },
}

/// Zig-zag sequence of quantized DCT coefficients
///
/// Figure A.6
pub static ZIGZAG: [u8; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10,
    17, 24, 32, 25, 18, 11, 4, 5,
    12, 19, 26, 33, 40, 48, 41, 34,
    27, 20, 13, 6, 7, 14, 21, 28,
    35, 42, 49, 56, 57, 50, 43, 36,
    29, 22, 15, 23, 30, 37, 44, 51,
    58, 59, 52, 45, 38, 31, 39, 46,
    53, 60, 61, 54, 47, 55, 62, 63,
];

pub(crate) struct JfifWriter<W: Write> {
    w: W,
}

impl<W: Write> JfifWriter<W> {
    pub fn new(w: W) -> Self {
        JfifWriter {
            w,
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> IOResult<()> {
        self.w.write_all(buf)
    }

    pub fn write_u8(&mut self, value: u8) -> IOResult<()> {
        self.w.write_all(&[value])
    }

    pub fn write_u16(&mut self, value: u16) -> IOResult<()> {
        self.w.write_all(&value.to_be_bytes())
    }

    pub fn write_marker(&mut self, marker: Marker) -> IOResult<()> {
        self.write(&[0xFF, marker.into()])
    }

    pub fn write_segment(&mut self, marker: Marker, data: &[u8]) -> IOResult<()> {
        self.write_marker(marker)?;
        self.write_u16(data.len() as u16 + 2)?;
        self.write(data)?;

        Ok(())
    }

    pub fn write_header(&mut self, density: &Density) -> IOResult<()> {
        self.write_marker(Marker::APP(0))?;
        self.write_u16(16)?;

        self.write(b"JFIF\0")?;
        self.write(&[0x01, 0x02])?;

        match *density {
            Density::None => {
                self.write_u8(0x00)?;
                self.write_u16(1)?;
                self.write_u16(1)?;
            }
            Density::Inch { x, y } => {
                self.write_u8(0x01)?;
                self.write_u16(x)?;
                self.write_u16(y)?;
            }
            Density::Centimeter { x, y } => {
                self.write_u8(0x02)?;
                self.write_u16(x)?;
                self.write_u16(y)?;
            }
        }

        self.write(&[0x00, 0x00])
    }

    /// Append huffman table segment
    ///
    /// - `class`: 0 for DC or 1 for AC
    /// - `dest`: 0 for luma or 1 for chroma tables
    ///
    /// Layout:
    /// ```txt
    /// |--------|---------------|--------------------------|--------------------|--------|
    /// | 0xFFC4 | 16 bit length | 4 bit class / 4 bit dest |  16 byte num codes | values |
    /// |--------|---------------|--------------------------|--------------------|--------|
    /// ```
    ///
    pub fn write_huffman_segment(&mut self, class: CodingClass, destination: u8, table: &HuffmanTable) -> IOResult<()> {
        assert!(destination < 4, "Bad destination: {}", destination);

        self.write_marker(Marker::DHT)?;
        self.write_u16(2 + 1 + 16 + table.values().len() as u16)?;

        self.write_u8(((class as u8) << 4) | destination as u8)?;
        self.write(table.length())?;
        self.write(table.values())?;

        Ok(())
    }

    /// Append a quantization table
    ///
    /// - `precision`: 0 which means 1 byte per value.
    /// - `dest`: 0 for luma or 1 for chroma tables
    ///
    /// Layout:
    /// ```txt
    /// |--------|---------------|------------------------------|--------|--------|-----|--------|
    /// | 0xFFDB | 16 bit length | 4 bit precision / 4 bit dest | V(0,0) | V(0,1) | ... | V(7,7) |
    /// |--------|---------------|------------------------------|--------|--------|-----|--------|
    /// ```
    ///
    pub fn write_quantization_segment(&mut self, destination: u8, table: &QuantizationTable) -> IOResult<()> {
        assert!(destination < 4, "Bad destination: {}", destination);

        self.write_marker(Marker::DQT)?;
        self.write_u16(2 + 1 + 64)?;

        self.write_u8(destination as u8)?;

        for &v in &ZIGZAG {
            self.write_u8(table.get(v as usize))?;
        }

        Ok(())
    }
}

