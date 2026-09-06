/* Mounts the shader named in window.DEMO_KEY behind the hero.
   Handles the theme swap, reduced motion, and pausing when off screen.

   Four of the ten shaders take a noise texture as a uniform. The library hands
   it over as an Image with a data URL, which is still decoding on the first
   tick, and mounting with an undecoded image throws. So every image uniform is
   awaited before the shader is built. */
(function () {
  var root = document.documentElement;
  var key = window.DEMO_KEY;
  var host = document.getElementById("shader-host");
  var cfg = window.SHADERS && window.SHADERS[key];
  var preset = window.DEMOS && window.DEMOS[key];
  var mount = null;
  var reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  function say(why) {
    var s = document.getElementById("shader-state");
    if (s) s.textContent = why;
  }

  function fail(why) {
    host.classList.add("no-gl");
    say(why);
  }

  function theme() {
    return root.getAttribute("data-theme") === "light" ? "light" : "dark";
  }

  function paramsFor(t) {
    var p = Object.assign({}, cfg.defaults, preset[t]);
    delete p.dim;
    return p;
  }

  function speedOf(p) {
    if (reduced) return 0;
    return p.speed === undefined ? 1 : p.speed;
  }

  function applyDim(t) {
    var d = preset[t].dim;
    host.style.opacity = d === undefined ? 1 : d;
  }

  // Resolve every HTMLImageElement sitting in the uniforms before mounting.
  function readyImages(uniforms) {
    var waits = [];
    for (var k in uniforms) {
      var v = uniforms[k];
      if (v && v.tagName === "IMG" && !v.complete) {
        waits.push(new Promise(function (done) {
          v.addEventListener("load", done, { once: true });
          v.addEventListener("error", done, { once: true });
        }));
      }
    }
    return waits.length ? Promise.all(waits) : Promise.resolve();
  }

  function start() {
    if (!cfg || !preset) { fail("This shader failed to load."); return; }
    if (!document.createElement("canvas").getContext("webgl2")) {
      fail("This browser has no WebGL2, so the page falls back to a plain gradient.");
      return;
    }
    var p = paramsFor(theme());
    var uniforms;
    try {
      uniforms = cfg.build(p);
    } catch (e) {
      fail("Building the uniforms failed: " + (e && e.message));
      return;
    }
    readyImages(uniforms).then(function () {
      try {
        mount = new window.PaperShaders.ShaderMount(
          host, cfg.frag, uniforms, undefined, speedOf(p), 0, 1, 1920 * 1080);
      } catch (e) {
        fail("This shader failed to start: " + (e && e.message));
        return;
      }
      applyDim(theme());
      if (reduced) mount.setFrame(2500);
      watch();
    });
  }

  function retheme() {
    if (!mount) return;
    var t = theme();
    var p = paramsFor(t);
    var u = cfg.build(p);
    readyImages(u).then(function () {
      mount.setUniforms(u);
      mount.setSpeed(speedOf(p));
      applyDim(t);
      if (reduced) mount.setFrame(2500);
    });
  }

  // Stop the loop while the hero is scrolled away or the tab is hidden.
  function watch() {
    if (reduced || !("IntersectionObserver" in window)) return;
    var visible = true, running = true;
    function sync() {
      var want = visible && !document.hidden;
      if (want === running) return;
      running = want;
      mount.setSpeed(want ? speedOf(paramsFor(theme())) : 0);
    }
    new IntersectionObserver(function (es) {
      visible = es[0].isIntersecting;
      sync();
    }, { threshold: 0 }).observe(host);
    document.addEventListener("visibilitychange", sync);
  }

  start();
  document.addEventListener("fyf-theme", retheme);
})();
