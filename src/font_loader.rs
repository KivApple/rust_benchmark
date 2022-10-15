use std::cell::RefCell;
use std::io::{Cursor, Error, ErrorKind};
use ahash::AHashMap as HashMap;
use byteorder::{BigEndian, ReadBytesExt};
use glam::Vec2;
use rgb::RGBA8;
use std::mem::size_of;

#[derive(Debug)]
pub struct FontGlyph {
	tex_coord: Vec2,
	tex_size: Vec2,
	offset: Vec2,
	size: Vec2,
	width: f32
}

pub struct PF2Loader<'a> {
	data: &'a [u8],
	cursor: RefCell<Cursor<&'a [u8]>>,
	section_type: u32,
	section_len: usize,
	section_start: usize,
	point_size: u16,
	max_width: u16,
	max_height: u16,
	ascent: u16,
	descent: u16,
	character_index: HashMap<u32, (u32, usize)>,
	col_count: usize,
	texture_width: usize,
	texture_height: usize,
	texture_data: RefCell<Vec<RGBA8>>,
	glyphs: HashMap<u32, FontGlyph>
}

struct PF2CharDef {
	width: u16,
	height: u16,
	x_offset: i16,
	y_offset: i16,
	device_width: i16
}

impl PF2Loader<'_> {
	pub fn new(data: &[u8]) -> PF2Loader {
		PF2Loader { 
			data,
			cursor: RefCell::new(Cursor::new(data)),
			section_type: 0,
			section_len: 0,
			section_start: 0,
			point_size: 0,
			max_width: 0,
			max_height: 0,
			ascent: 0,
			descent: 0,
			character_index: HashMap::new(),
			col_count: 0,
			texture_width: 0,
			texture_height: 0,
			texture_data: RefCell::new(Vec::new()),
			glyphs: HashMap::new()
		}
	}

	fn make_section_type(t: &[u8; 4]) -> u32 {
		return (t[0] as u32) << 24 | (t[1] as u32) << 16 | (t[2] as u32) << 8 | (t[3] as u32)
	}

	fn section_type_as_str(&self) -> Result<&str, Error> {
		std::str::from_utf8(
			&self.data[self.section_start - 8..self.section_start - 4]
		).map_err(|e| Error::new(ErrorKind::Other, e))
	}

	fn read_section(&mut self) -> Result<bool, Error> {
		let cursor = self.cursor.get_mut();
		cursor.set_position((self.section_start + self.section_len) as u64);
		self.section_type = cursor.read_u32::<BigEndian>()?;
		self.section_len = cursor.read_u32::<BigEndian>()? as usize;
		self.section_start = cursor.position() as usize;
		Ok(self.section_len < 0xFFFFFFFF)
	}

	fn section_as_str(&self) -> Result<&str, Error> {
		std::str::from_utf8(&self.data[self.section_start..self.section_start + self.section_len])
			.map_err(|e| Error::new(ErrorKind::Other, e))
	}

	fn parse_character_index(&mut self) -> Result<(), Error> {
		if !self.character_index.is_empty() {
			return Err(Error::new(ErrorKind::Other, "Character index occured more than once"));
		}
		const RECORD_LEN: usize = size_of::<u32>() + size_of::<u8>() + size_of::<u32>();
		if self.section_len % RECORD_LEN != 0 {
			return Err(Error::new(ErrorKind::Other, "Character index length is not divisible by a record size"));
		}
		let count = self.section_len / RECORD_LEN;
		self.character_index.reserve(count);
		let cursor = self.cursor.get_mut();
		for _ in 0..count {
			let unicode_code_point = cursor.read_u32::<BigEndian>()?;
			let flags = cursor.read_u8()?;
			let offset = cursor.read_u32::<BigEndian>()?;
			assert!(flags & 0b111 == 0);
			self.character_index.insert(unicode_code_point, (offset, self.character_index.len()));
		}
		//trace!("Character index contains {} items", self.character_index.len());
		Ok(())
	}

	fn parse_section(&mut self) -> Result<(), Error> {
		let cursor = self.cursor.get_mut();
		if self.section_type == Self::make_section_type(b"NAME") {
			//trace!("Font name: {}", self.section_as_str()?);
		} else if self.section_type == Self::make_section_type(b"FAMI") {
			//trace!("Font family: {}", self.section_as_str()?);
		} else if self.section_type == Self::make_section_type(b"WEIG") {
			//trace!("Font weight: {}", self.section_as_str()?);
		} else if self.section_type == Self::make_section_type(b"SLAN") {
			//trace!("Font slant: {}", self.section_as_str()?);
		} else if self.section_type == Self::make_section_type(b"PTSZ") {
			self.point_size = cursor.read_u16::<BigEndian>()?;
			//trace!("Point size: {}", self.point_size);
		} else if self.section_type == Self::make_section_type(b"MAXW") {
			self.max_width = cursor.read_u16::<BigEndian>()?;
			//trace!("Max width: {}", self.max_width);
		} else if self.section_type == Self::make_section_type(b"MAXH") {
			self.max_height = cursor.read_u16::<BigEndian>()?;
			//trace!("Max height: {}", self.max_height);
		} else if self.section_type == Self::make_section_type(b"ASCE") {
			self.ascent = cursor.read_u16::<BigEndian>()?;
			//trace!("Ascent: {}", self.ascent);
		} else if self.section_type == Self::make_section_type(b"DESC") {
			self.descent = cursor.read_u16::<BigEndian>()?;
			//trace!("Descent: {}", self.descent);
		} else if self.section_type == Self::make_section_type(b"CHIX") {
			self.parse_character_index()?;
		} else {
			//warn!("Unknown section: {}", self.section_type_as_str()?);
		}
		Ok(())
	}

	fn read_char_def(&self) -> Result<PF2CharDef, Error> {
		let mut cursor = self.cursor.borrow_mut();
		let width = cursor.read_u16::<BigEndian>()?;
		let height = cursor.read_u16::<BigEndian>()?;
		let x_offset = cursor.read_i16::<BigEndian>()?;
		let y_offset = cursor.read_i16::<BigEndian>()?;
		let device_width = cursor.read_i16::<BigEndian>()?;
		Ok(PF2CharDef { width, height, x_offset, y_offset, device_width })
	}

	fn parse_char_bitmap(&self, index: usize, def: &PF2CharDef) -> FontGlyph {
		let x0 = (index % self.col_count) * self.max_width as usize;
		let y0 = (index / self.col_count) * self.max_height as usize;
		let mut texture_data = self.texture_data.borrow_mut();
		let base = self.cursor.borrow().position() as usize;
		for y in 0..def.height as usize {
			let j = (y0 + y) * self.texture_width + x0;
			for x in 0..def.width as usize {
				let i = y * def.width as usize + x;
				let byte = self.data[base + i / 8];
				if byte & (1 << (7 - i % 8)) != 0 {
					texture_data[j + x] = RGBA8::new(255, 255, 255, 255);
				}
			}
		}
		FontGlyph { 
			tex_coord: Vec2::new(x0 as f32 / self.texture_width as f32, y0 as f32 / self.texture_height as f32), 
			tex_size: Vec2::new(def.width as f32 / self.texture_width as f32, def.height as f32 / self.texture_height as f32), 
			offset: Vec2::new(def.x_offset as f32 / self.point_size as f32, def.y_offset as f32 / self.point_size as f32), 
			size: Vec2::new(def.width as f32 / self.point_size as f32, def.height as f32 / self.point_size as f32), 
			width: def.device_width as f32 / self.point_size as f32
		}
	}

	fn parse_data_section(&mut self) -> Result<(), Error> {
		if self.character_index.is_empty() {
			return Err(Error::new(ErrorKind::Other, "Character index is empty"));
		}
		if self.max_width == 0 {
			return Err(Error::new(ErrorKind::Other, "Max width is unspecified or zero"));
		}
		if self.max_height == 0 {
			return Err(Error::new(ErrorKind::Other, "Max height is unspecified or zero"));
		}
		self.col_count = (self.character_index.len() as f32 * self.max_height as f32 / self.max_width as f32).sqrt().ceil() as usize;
		self.texture_width = self.col_count * self.max_width as usize;
		self.texture_height = (self.character_index.len() + self.col_count - 1) / self.col_count * self.max_height as usize;
		self.texture_data.get_mut().resize(self.texture_width * self.texture_height, RGBA8::default());
		self.glyphs.reserve(self.character_index.len());
		for (unicode_code_point, (offset, index)) in &self.character_index {
			self.cursor.get_mut().set_position(*offset as u64);
			let def = self.read_char_def()?;
			let glyph = self.parse_char_bitmap(*index, &def);
			self.glyphs.insert(*unicode_code_point, glyph);
		}
		Ok(())
	}

	pub fn load(&mut self) -> Result<(Vec<RGBA8>, HashMap<u32, FontGlyph>), Error> {
		self.read_section()?;
		if self.section_type != Self::make_section_type(b"FILE") {
			return Err(Error::new(ErrorKind::Other, format!("Expected \"FILE\" section, but \"{}\" found", self.section_type_as_str()?)));
		}
		if self.section_as_str()? != "PFF2" {
			return Err(Error::new(
				ErrorKind::Other, 
				format!(
					"FILE section contents must be equal to \"PFF2\", but \"{}\" found", 
					self.section_as_str()?
				)
			));
		}
		while self.read_section()? {
			self.parse_section()?;
		}
		if self.section_type != Self::make_section_type(b"DATA") {
			return Err(Error::new(ErrorKind::Other, format!("Expected \"DATA\" section, but \"{}\" found", self.section_type_as_str()?)));
		}
		self.parse_data_section()?;
        let mut texture_data = Vec::<RGBA8>::new();
        std::mem::swap(self.texture_data.get_mut(), &mut texture_data);
		let mut glyphs = HashMap::<u32, FontGlyph>::new();
		std::mem::swap(&mut self.glyphs, &mut glyphs);
		Ok((texture_data, glyphs))
	}
}
