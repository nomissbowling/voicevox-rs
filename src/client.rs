//! voicevox client
//!

use std::error::Error;
use std::io::{Cursor, Read, BufReader, BufWriter};
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs;
use std::thread;
use std::time;

use aho_corasick::AhoCorasick; // {AhoCorasick, PatternID};
use csv::{ReaderBuilder, WriterBuilder};

use rodio::{Sink, OutputStream, OutputStreamHandle, Decoder, source::Source};

use reqwest;
use reqwest::header::{CONTENT_TYPE, CONNECTION};

use serde_urlencoded;
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

/// tsv object Words
#[derive(Serialize, Deserialize, Debug)]
pub struct Words {
  /// before
  pub before: String,
  /// after
  pub after: String,
  /// ext
  #[serde(rename = "extend")]
  pub ext: i32
}

/// VOICEVOX Client
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
  pub hm: HashMap<i32, (usize, usize)>,
  /// ac_tpl
  pub ac_tpl: Option<(AhoCorasick, Vec<String>)>,
  /// words (String, String)
  pub words: BTreeMap<i32, (String, String)>,
  /// words file
  pub words_file: String,
  /// cfg path
  pub cfg_path: String,
  /// stream_tpl
  pub stream_tpl: Option<(OutputStream, OutputStreamHandle)>,
  /// sink
  pub sink: Option<Sink>
}

/// VOICEVOX Client implementation
impl VVClient {

/// constructor VOICEVOX Client
pub fn new() -> VVClient {
  let mut vvc = VVClient{host: "127.0.0.1".to_string(), port: 50021,
    speakers: vec![],
    dct: HashMap::<String, usize>::new(),
    hm: HashMap::<i32, (usize, usize)>::new(),
    ac_tpl: None,
    words: BTreeMap::<i32, (String, String)>::new(),
    words_file: "words.tsv".to_string(),
    cfg_path: "./cfg".to_string(),
    stream_tpl: None,
    sink: None};
  vvc.get_speakers().unwrap();
  for (idx, speaker) in vvc.speakers.iter().enumerate() {
    vvc.dct.insert(speaker.name.clone(), idx);
    for (i, style) in speaker.styles.iter().enumerate() {
      vvc.hm.insert(style.id, (idx, i));
    }
  }
  vvc.load_words().unwrap();
  let mut patterns = Vec::<String>::new();
  let mut replace_with = Vec::<String>::new();
  for (_ext, (before, after)) in &vvc.words {
    patterns.push(before.clone());
    replace_with.push(after.clone());
  }
  vvc.ac_tpl = Some((AhoCorasick::new(patterns).unwrap(), replace_with));
  vvc.stream_tpl = Some(OutputStream::try_default().unwrap());
  match &vvc.stream_tpl {
  None => (),
  Some((_stream, stream_handle)) => {
    vvc.sink = Some(Sink::try_new(stream_handle).unwrap());
  }
  }
  vvc
}

/// path to words
pub fn words_path(&self) -> Result<String, Box<dyn Error>> {
  let bp = PathBuf::from(self.cfg_path.as_str());
  Ok(bp.join(self.words_file.as_str()).to_str().ok_or("not path")?.to_string())
}

/// load words (called inner constructor)
pub fn load_words(&mut self) -> Result<(), Box<dyn Error>> {
  let tsv = BufReader::new(fs::File::open(self.words_path()?.as_str())?);
  let mut rdr = ReaderBuilder::new().delimiter(b'\x09').from_reader(tsv);
/*
  if let Some(result) = rdr.records().next() {
    let rec = result?;
    assert_eq!(rec, vec!["/", "スラッシュ", "0"]); // always StringRecord
    Ok(())
  }else{
    Err(From::from("expected at least one record but got none"))
  }
*/
  for result in rdr.deserialize() { // rdr.records() is always StringRecord
    let rec: Words = result?;
    self.words.insert(rec.ext, (rec.before, rec.after));
  }
  Ok(())
}

/// save words
pub fn save_words(&self) -> Result<(), Box<dyn Error>> {
  // open not create: Err(Os{code: 5, kind: PermissionDenied, message: "..."})
  let tsv = BufWriter::new(fs::File::create(self.words_path()?.as_str())?);
  let mut wtr = WriterBuilder::new().delimiter(b'\x09').from_writer(tsv);
  // wtr.write_record(&["before", "after", "extend"])?; // needless
  for (ext, (before, after)) in &self.words {
    let rec = Words{before: before.clone(), after: after.clone(), ext: *ext};
    wtr.serialize(rec)?;
  }
  wtr.flush()?;
  Ok(())
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
  let t = match &self.ac_tpl {
  None => txt.to_string(),
  Some((ac, replace_with)) => ac.replace_all(txt, replace_with)
  };
  let s = format!("{}", id);
  let m: HashMap<&str, &str> = vec![
    ("text", t.as_str()), ("speaker", s.as_str())].into_iter().collect();
  let params = serde_urlencoded::to_string(m)?;
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
    .header(CONTENT_TYPE, "application/json")
    .header(CONNECTION, "close")
    .body(qs).send()?;
  let mut buf = Vec::<u8>::new();
  resp.read_to_end(&mut buf).expect("Failed to read response");
  Ok(buf)
}

/// speak
pub fn speak(&self, dat: Vec<u8>, sec: u64) -> Result<(), Box<dyn Error>> {
  // let reader = BufReader::new(fs::File::open("biwa_UAC.ogg")?);
  let reader = Cursor::new(dat);
  let source = Decoder::new(reader)?;
  if sec > 0 {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    stream_handle.play_raw(source.convert_samples())?;
    thread::sleep(time::Duration::from_secs(sec)); // need to keep main
    Ok(())
  }else{
    match &self.sink {
    None => Err("sink is not initialized".into()),
    Some(sink) => {
      sink.append(source.convert_samples::<f32>());
      Ok(())
    }
    }
  }
}

/// speak_flush
pub fn speak_flush(&self) -> Result<(), Box<dyn Error>> {
  match &self.sink {
  None => Err("sink is not initialized".into()),
  Some(sink) => {
    sink.sleep_until_end();
    Ok(())
  }
  }
}

}
