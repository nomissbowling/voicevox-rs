voicevox-rs
===========

VOICEVOX Client library for Rust


```rust
let vvc = VVClient::new();
vvc.display_speakers().unwrap();
let Some(id) = vvc.speaker("ずんだもん", "ノーマル") else { panic!("id") };
let qs = vvc.query("ずんだもんなのだ。", id).unwrap();
let dat = vvc.synth(qs, id).unwrap();
vvc.speak(dat, 3).unwrap();
```


```rust
let vvc = VVClient::new();
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
  vvc.speak(dat, 3).unwrap();
  ps.speedScale = 1.5;
  let dat = vvc.synth(vvc.phrases_to_str(&ps).unwrap(), id).unwrap();
  vvc.speak(dat, 3).unwrap();
}
```


Requirements
============

- [VOICEVOX (v0.14.7)]( https://voicevox.hiroshiba.jp/ )


License
=======

MIT

