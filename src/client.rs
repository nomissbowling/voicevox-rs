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
  /// permitted_synthesis_morphing
  pub permitted_synthesis_morphing: String
}

/// json object inner Speaker
#[derive(Serialize, Deserialize, Debug)]
pub struct Style {
  /// id
  pub id: i32,
  /// name
  pub name: String
}

/// json object Speaker
#[derive(Serialize, Deserialize, Debug)]
pub struct Speaker {
  /// name
  pub name: String,
  /// speaker_uuid
  pub speaker_uuid: String,
  /// styles
  pub styles: Vec<Style>,
  /// supported_features
  pub supported_features: Feat,
  /// version
  pub version: String
}

/// json null as Option
fn deserialize_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
  where T: Default + Deserialize<'de>, D: Deserializer<'de>
{
  Deserialize::deserialize(deserializer)
}

/// json object inner Phrase
#[derive(Serialize, Deserialize, Debug)]
pub struct Mora {
  /// consonant (allow null)
  #[serde(deserialize_with = "deserialize_null")]
  pub consonant: Option<String>,
  /// consonant_length (allow null)
  #[serde(deserialize_with = "deserialize_null")]
  pub consonant_length: Option<f64>,
  /// pitch
  pub pitch: f64,
  /// text
  pub text: String,
  /// vowel
  pub vowel: String,
  /// vowel_length
  pub vowel_length: f64
}

/// json object inner Phrases
#[derive(Serialize, Deserialize, Debug)]
pub struct Phrase {
  /// accent
  pub accent: i32,
  /// is_interrogative
  pub is_interrogative: bool,
  /// moras
  pub moras: Vec<Mora>,
  /// pause_mora (allow null)
  #[serde(deserialize_with = "deserialize_null")]
  pub pause_mora: Option<String>
}

/// json object Phrases
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Phrases {
  /// accent_phrases
  pub accent_phrases: Vec<Phrase>,
  /// intonationScale
  pub intonationScale: f64,
  /// kana
  pub kana: String,
  /// outputSamplingRate
  pub outputSamplingRate: i32,
  /// outputStereo
  pub outputStereo: bool,
  /// pitchScale
  pub pitchScale: f64,
  /// postPhonemeLength
  pub postPhonemeLength: f64,
  /// prePhonemeLength
  pub prePhonemeLength: f64,
  /// speedScale
  pub speedScale: f64,
  /// volumeScale
  pub volumeScale: f64
}

/// VOICEVOX Client
#[derive(Debug)]
pub struct VVClient {
  /// host
  pub host: String,
  /// port
  pub port: i32,
  /// speakers
  pub speakers: Vec<Speaker>,
  /// dct (name, index)
  pub dct: HashMap<String, usize>,
  /// hm (speaker_id, (index of speakers, index of styles))
  pub hm: HashMap<i32, (usize, usize)>
}

/// VOICEVOX Client implementation
impl VVClient {

/// constructor VOICEVOX Client
pub fn new() -> VVClient {
  let mut vvc = VVClient{host: "127.0.0.1".to_string(), port: 50021,
    speakers: vec![],
    dct: HashMap::<String, usize>::new(),
    hm: HashMap::<i32, (usize, usize)>::new() };
  vvc.get_speakers().unwrap();
  for (idx, speaker) in vvc.speakers.iter().enumerate() {
    vvc.dct.insert(speaker.name.clone(), idx);
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

/// find speaker idx
pub fn find(&self, name: &str) -> Option<usize> {
  match self.dct.get(name) {
  None => None,
  Some(idx) => Some(*idx)
  }
}

/// find speaker id
pub fn speaker(&self, name: &str, ss: &str) -> Option<i32> {
  match self.find(name) {
  None => None,
  Some(idx) => {
    for style in &self.speakers[idx].styles {
      if style.name == ss { return Some(style.id) }
    }
    None
  }
  }
}

/// get speaker name
pub fn name(&self, idx: usize) -> &String {
  &self.speakers[idx].name
}

/// get style name
pub fn style_name(&self, idx: (usize, usize)) -> &String {
  &self.speakers[idx.0].styles[idx.1].name
}

/// get style id (speaker id)
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
  let params = format!("text={}&speaker={}", txt, id); // must be url encoded
  let mut resp = cli.post(format!("{}?{}", uri, params)).send()?;
  let mut buf = String::new();
  resp.read_to_string(&mut buf).expect("Failed to read response");
  Ok(buf)
}

/// audio query string to json object Phrases
pub fn phrases(&self, qs: &String) -> Result<Phrases, Box<dyn Error>> {
  Ok(serde_json::from_str::<Phrases>(qs).expect("parse error"))
}

/// json object Phrases to string
pub fn phrases_to_str(&self, ps: &Phrases) -> Result<String, Box<dyn Error>> {
  Ok(serde_json::to_string(ps)?)
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
pub fn speak(&self, dat: Vec<u8>, sec: u64) -> Result<(), Box<dyn Error>> {
  let (_stream, stream_handle) = OutputStream::try_default()?;
/*
  let reader = std::io::BufReader::new(
    std::fs::File::open("biwa_UAC.ogg")?);
*/
  let reader = Cursor::new(dat);
  let source = Decoder::new(reader)?;
  stream_handle.play_raw(source.convert_samples())?;
  std::thread::sleep(std::time::Duration::from_secs(sec)); // need to keep main
  Ok(())
}

}
