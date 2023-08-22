voicevox-rs
===========

VOICEVOX Client library for Rust


```
let vvc = VVClient::new();
let id = 3;
let qs = vvc.query("ずんだもんなのだ。", id).unwrap();
let dat = vvc.synth(qs, id).unwrap();
vvc.speak(dat).unwrap();
```


Requirements
============

- [VOICEVOX (v0.14.7)]( https://voicevox.hiroshiba.jp/ )


License
=======

MIT

