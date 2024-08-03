/// A cosmetic effect on a tile, which can gradually fade out.
#[derive(Clone, Debug)]
pub struct TileEffect {
	pub x: i32,
	pub y: i32,
	pub z: i32,
	pub alpha: u8,
	pub texture: EffectType,
}

#[derive(Clone, Debug)]
pub enum EffectType {
	Red,
	Lime,
}
