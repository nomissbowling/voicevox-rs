#![doc(html_root_url = "https://docs.rs/voicevox-rs/0.4.7")]
//! voicevox client library for Rust
//!
//! # Requirements
//!
//! - [ VOICEVOX ]( https://voicevox.hiroshiba.jp/ )
//!

pub mod client;

/**/

/// to see printed: with option -- --show-output
#[cfg(test)]
mod tests {
  use super::client::VVClient;

  /// check speakers and speaker id for VOICEVOX 0.14.7
  #[test]
  fn check_speakers() {
    let vvc = VVClient::new();
    vvc.display_speakers().unwrap();
    let ptn: Vec<(i32, &str, usize, usize, &str)> = vec![
      (46, "小夜/SAYO", 17, 0, "ノーマル"),
      (0, "四国めたん", 0, 1, "あまあま"),
      (3, "ずんだもん", 1, 0, "ノーマル")];
    for p in ptn {
      assert_eq!(vvc.speaker(p.1, p.4), Some(p.0));
      assert_eq!(vvc.find(p.1), Some(p.2));
      assert_eq!(vvc.name(p.2), p.1);
      assert_eq!(vvc.style_id((p.2, p.3)), p.0);
      assert_eq!(vvc.style_name((p.2, p.3)), p.4);
      assert_eq!(vvc.detail_speaker(p.0),
        (true, format!("{} {} {}", p.0, p.1, p.4)));
    }
  }

  /// check query for VOICEVOX 0.14.7
  #[test]
  fn check_query() {
    let vvc = VVClient::new();
    let Some(id) = vvc.speaker("ずんだもん", "ノーマル") else { panic!("id") };
    let qs = vvc.query("ずんだもんなのだ。", id).unwrap();
    let ps = vvc.phrases(&qs).unwrap();
    // println!("{:?}", ps);
    assert_eq!(ps.kana, "ズ'ンダモンナ/ノダ'");
  }

  /// check synthesis for VOICEVOX 0.14.7
  #[test]
  fn check_synthesis() {
    let vvc = VVClient::new();
    let Some(id) = vvc.speaker("ずんだもん", "ノーマル") else { panic!("id") };
    let qs = vvc.query("ずんだもんなのだ。", id).unwrap();
    let mut ps = vvc.phrases(&qs).unwrap();
    let dat = vvc.synth(qs, id).unwrap();
    println!("response: {}", dat.len()); // 69676 = 8 + 69668 (RIFF size)
    // println!("response: {}", String::from_utf8(dat).unwrap()); // if error
    let cmp: Vec<u8> = vec![
      0x52, 0x49, 0x46, 0x46, 0x24, 0x10, 0x01, 0x00, // RIFF 69668 = 00011024
      0x57, 0x41, 0x56, 0x45, // WAVE
      0x66, 0x6d, 0x74, 0x20, // 'fmt '
      0x10, 0x00, 0x00, 0x00, // chunk size (0x00000010 linear PCM) 0x12 0x28
      0x01, 0x00, // format (1 no compress) 1 3 6 7 -2
      0x01, 0x00, // channels
      0xc0, 0x5d, 0x00, 0x00, // samples/sec (blocks/sec frequency) (24000)
      0x80, 0xbb, 0x00, 0x00, // avg bytes/sec (48000)
      0x02, 0x00, // block size
      0x10, 0x00, // bits/sample
      // [(2) ext parameter size]
      // [(n) ext parameters]
      0x64, 0x61, 0x74, 0x61, // 'data' sub chunk
      0x00, 0x10, 0x01, 0x00]; // sub chunk size (+ WAVE head 0x24 = RIFF size)
      // ... wav data
    assert_eq!(dat[0..cmp.len()], cmp);
    assert_eq!(vvc.speak(dat, 2).unwrap(), ());
    // change speed to speak
    ps.speedScale = 1.5;
    let dat = vvc.synth(vvc.phrases_to_str(&ps).unwrap(), id).unwrap();
    assert_eq!(vvc.speak(dat, 3).unwrap(), ());
  }

  /// check words for VOICEVOX 0.14.7
  #[test]
  fn check_words() {
    let vvc = VVClient::new();
    assert_eq!(vvc.save_words().unwrap(), ());
    let characters: Vec<(&str, &str)> = vec![
      ("ずんだもん", "あまあま"),
      ("波音リツ", "クイーン"),
      ("波音リツ", "ノーマル"),
      ("小夜/SAYO", "ノーマル")];
    for chara in characters {
      let Some(id) = vvc.speaker(chara.0, chara.1) else { panic!("id") };
      // dangerous sequence * and / in the comment
      let qs = vvc.query(format!("{} {} ＋+－-×*÷/ 🐼🐼🍞🍞🐼🍞🐼",
        chara.0, chara.1).as_str(), id).unwrap();
      let dat = vvc.synth(qs, id).unwrap();
      assert_eq!(vvc.speak(dat, 0).unwrap(), ());
    }
    assert_eq!(vvc.speak_flush().unwrap(), ()); // must call it after speak 0
  }

  /// check characters for VOICEVOX 0.14.7
  #[test]
  fn check_characters() {
    let vvc = VVClient::new();
    // vvc.display_speakers().unwrap();
    let characters: Vec<(&str, &str)> = vec![
      ("雨晴はう", "ノーマル"),
      ("春日部つむぎ", "ノーマル"),
      ("四国めたん", "ノーマル"),
      ("四国めたん", "あまあま"),
      ("ずんだもん", "ノーマル"),
      ("小夜/SAYO", "ノーマル"),
      ("猫使アル", "ノーマル"),
      ("猫使アル", "おちつき"),
      ("猫使アル", "うきうき"),
      ("猫使ビィ", "ノーマル"),
      ("猫使ビィ", "おちつき"),
      ("猫使ビィ", "人見知り")];
    for chara in characters {
      let Some(id) = vvc.speaker(chara.0, chara.1) else { panic!("id") };
      let qs = vvc.query(format!("{}です{}なのです",
        chara.0, chara.1).as_str(), id).unwrap();
      let mut ps = vvc.phrases(&qs).unwrap();
      let dat = vvc.synth(qs, id).unwrap();
      assert_eq!(vvc.speak(dat, 0).unwrap(), ());
      ps.speedScale = 1.5;
      let dat = vvc.synth(vvc.phrases_to_str(&ps).unwrap(), id).unwrap();
      assert_eq!(vvc.speak(dat, 0).unwrap(), ());
    }
    assert_eq!(vvc.speak_flush().unwrap(), ()); // must call it after speak 0
  }
}

/**/
