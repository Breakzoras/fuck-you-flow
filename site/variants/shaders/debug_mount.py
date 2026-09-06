"""Ask each failing shader to mount and report the real exception."""
import sys
import time

sys.path.insert(0, ".")
import shoot

PROBE = """
var key = arguments[0];
try {
  var cfg = window.SHADERS[key], pre = window.DEMOS[key];
  if (!cfg) return 'no config for ' + key;
  var p = Object.assign({}, cfg.defaults, pre.dark);
  delete p.dim;
  var u = cfg.build(p);
  var host = document.createElement('div');
  host.style.cssText = 'position:fixed;left:-9999px;width:200px;height:200px';
  document.body.appendChild(host);
  new window.PaperShaders.ShaderMount(host, cfg.frag, u, undefined, 0.4, 0, 1, 100000);
  return 'MOUNTED OK';
} catch (e) {
  return 'ERR: ' + (e && e.message ? e.message : e);
}
"""

KEYS = """
var key = arguments[0];
var cfg = window.SHADERS[key], pre = window.DEMOS[key];
var p = Object.assign({}, cfg.defaults, pre.dark);
delete p.dim;
var u = cfg.build(p);
var out = [];
for (var k in u) {
  var v = u[k];
  var t = v === undefined ? 'UNDEFINED' : (v === null ? 'null' :
          (Array.isArray(v) ? 'array[' + v.length + ']' : typeof v));
  out.push(k + ' = ' + t);
}
return out.join('\\n');
"""

CASES = [("s01-god-rays.html", "godRays"),
         ("s03-grain-gradient.html", "grainGradient"),
         ("s04-smoke-ring.html", "smokeRing"),
         ("s07-warp.html", "warp")]

for page, key in CASES:
    d = shoot.driver(900, 600)
    try:
        d.get(shoot.url(page))
        time.sleep(2)
        print("####", key)
        print(" ", d.execute_script(PROBE, key))
        bad = [ln for ln in d.execute_script(KEYS, key).split("\n") if "UNDEFINED" in ln]
        if bad:
            print("  undefined uniforms:", ", ".join(bad))
        for e in d.get_log("browser"):
            if e.get("level") == "SEVERE":
                print("  log:", e.get("message", "")[:300])
    finally:
        d.quit()
