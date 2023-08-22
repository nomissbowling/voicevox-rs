//! voicevox client
//!

use std::error::Error;
use std::io::{Cursor, Read}; // BufReader
use std::collections::HashMap;

use rodio::{OutputStream, Decoder, source::Source};

use reqwest;
use reqwest::header::CONTENT_TYPE;

use serde_json;
use serde_derive::{Serialize, Deserialize};
use serde::{Deserialize, Deserializer};

/// json object inner Speaker
#[derive(Serialize, Deserialize, Debug)]
pub struct Feat {
  pub permitted_synthesis_morphing: String
}

/// json object inner Speaker
#[derive(Serialize, Deserialize, Debug)]
pub struct Style {
  pub id: i32,
  pub name: String
}

/// json object Speaker
#[derive(Serialize, Deserialize, Debug)]
pub struct Speaker {
  pub name: String,
  pub speaker_uuid: String,
  pub styles: Vec<Style>,
  pub supported_features: Feat,
  pub version: String
}

/// json object inner Phrase
#[derive(Serialize, Deserialize, Debug)]
pub struct Mora {
  #[serde(deserialize_with = "deserialize_null_str")]
  pub consonant: String,
  #[serde(deserialize_with = "deserialize_null_f64")]
  pub consonant_length: f64,
  pub pitch: f64,
  pub text: String,
  pub vowel: String,
  pub vowel_length: f64
}

/// json null as String
fn deserialize_null_str<'de, D>(deserializer: D) -> Result<String, D::Error>
  where D: Deserializer<'de>
{
  // https://stackoverflow.com/questions/44205435/how-to-deserialize-a-json-file-which-contains-null-values-using-serde
/*
  let opt = Option::deserialize(deserializer)?;
  Ok(opt.unwrap_or_default())
*/
  Deserialize::deserialize(deserializer).map(|x: Option<_>| {
    x.unwrap_or("".to_string())
  })
}

/// json null as 0.0f64
fn deserialize_null_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
  where D: Deserializer<'de>
{
  Deserialize::deserialize(deserializer).map(|x: Option<_>| {
    x.unwrap_or(0.0f64)
  })
}

/// json object inner Phrases
#[derive(Serialize, Deserialize, Debug)]
pub struct Phrase {
  pub accent: i32,
  pub is_interrogative: bool,
  pub moras: Vec<Mora>,
  #[serde(deserialize_with = "deserialize_null")]
  pub pause_mora: Option<String>
}

/// json null as Option
fn deserialize_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
  where T: Default + Deserialize<'de>, D: Deserializer<'de>
{
  // https://komorinfo.com/blog/serde-option-deserialize/
  Deserialize::deserialize(deserializer)
}

/// json object Phrases
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Phrases {
  pub accent_phrases: Vec<Phrase>,
  pub intonationScale: f64,
  pub kana: String,
  pub outputSamplingRate: i32,
  pub outputStereo: bool,
  pub pitchScale: f64,
  pub postPhonemeLength: f64,
  pub prePhonemeLength: f64,
  pub speedScale: f64,
  pub volumeScale: f64
}

/// VOICEVOX Client
#[derive(Debug)]
pub struct VVClient {
  pub host: String,
  pub port: i32,
  pub speakers: Vec<Speaker>,
  pub hm: HashMap<i32, (usize, usize)>
}

/// VOICEVOX Client implementation
impl VVClient {

/// constructor VOICEVOX Client
pub fn new() -> VVClient {
  let mut vvc = VVClient{host: "127.0.0.1".to_string(), port: 50021,
    speakers: vec![], hm: HashMap::<i32, (usize, usize)>::new() };
  vvc.get_speakers().unwrap();
  for (idx, speaker) in vvc.speakers.iter().enumerate() {
    for (i, style) in speaker.styles.iter().enumerate() {
      vvc.hm.insert(style.id, (idx, i));
    }
  }
  vvc
}

/// get speakers (called inner constructor)
pub fn get_speakers(&mut self) -> Result<(), Box<dyn Error>> {
  let uri = format!("http://{}:{}/{}", self.host, self.port, "speakers");
  let cli = reqwest::blocking::Client::new();
  let mut resp = cli.get(uri).send()?;
  let mut buf = String::new();
  resp.read_to_string(&mut buf).expect("Failed to read response");
  self.speakers = serde_json::from_str(&buf).expect("parse error");
  Ok(())
}

/// display speaker names
pub fn display_speakers(&self) -> Result<(), Box<dyn Error>> {
  for (idx, speaker) in self.speakers.iter().enumerate() {
    println!("{}: {}", idx, speaker.name);
    for (i, style) in speaker.styles.iter().enumerate() {
      println!(" ({}, {}): {:?}", idx, i, style);
    }
  }
  Ok(())
}

/// get speaker name
pub fn name(&self, idx: usize) -> &String {
  &self.speakers[idx].name
}

/// get style name
pub fn style_name(&self, idx: (usize, usize)) -> &String {
  &self.speakers[idx.0].styles[idx.1].name
}

/// get style id
pub fn style_id(&self, idx: (usize, usize)) -> i32 {
  self.speakers[idx.0].styles[idx.1].id
}

/// detail speaker
pub fn detail_speaker(&self, id: i32) -> (bool, String) {
  match self.hm.get(&id) {
  None => (false, format!("{} UNKNOWN UNKNOWN", id)),
  Some((idx, i)) => (true, format!("{} {} {}",
    id, self.name(*idx), self.style_name((*idx, *i))))
  }
}

/// audio query as string
pub fn query(&self, txt: &str, id: i32) -> Result<String, Box<dyn Error>> {
  let uri = format!("http://{}:{}/{}", self.host, self.port, "audio_query");
  let cli = reqwest::blocking::Client::new();
  let params = format!("text={}&speaker={}", txt, id);
  let mut resp = cli.post(format!("{}?{}", uri, params)).send()?;
  let mut buf = String::new();
  resp.read_to_string(&mut buf).expect("Failed to read response");
  Ok(buf)
}

/// audio query string to json object Phrases
pub fn get_phrases(&self, qs: String) -> Result<Phrases, Box<dyn Error>> {
  Ok(serde_json::from_str::<Phrases>(&qs).expect("parse error"))
}

/// synthesis
pub fn synth(&self, qs: String, id: i32) -> Result<Vec<u8>, Box<dyn Error>> {
  let uri = format!("http://{}:{}/{}", self.host, self.port, "synthesis");
  let cli = reqwest::blocking::Client::new();
  let params = format!("speaker={}", id);
  let mut resp = cli.post(format!("{}?{}", uri, params))
    .header(CONTENT_TYPE, "application/json").body(qs).send()?;
  let mut buf = Vec::<u8>::new();
  resp.read_to_end(&mut buf).expect("Failed to read response");
  Ok(buf)
}

/// speak
pub fn speak(&self, dat: Vec<u8>) -> Result<(), Box<dyn Error>> {
  let (_stream, stream_handle) = OutputStream::try_default()?;
/*
  let reader = std::io::BufReader::new(
    std::fs::File::open("biwa_UAC.ogg")?);
*/
  let reader = Cursor::new(dat);
  let source = Decoder::new(reader)?;
  stream_handle.play_raw(source.convert_samples())?;
  std::thread::sleep(std::time::Duration::from_secs(3)); // need to keep main
  Ok(())
}

}
