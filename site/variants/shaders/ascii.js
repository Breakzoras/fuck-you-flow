/* A real ASCII renderer for the Fuck You Flow hero.

   The page draws a moving field of light, then reads it back one character cell
   at a time: the brighter the cell, the denser the character printed there. The
   characters are the page's own monospace face, baked into a texture at load
   time, so what you see is genuine type rather than a pattern that resembles it.

   Self contained: one canvas, WebGL1, no library, no external asset. */
window.FYFAscii = (function () {
  "use strict";

  var VERT =
    "attribute vec2 a;void main(){gl_Position=vec4(a,0.,1.);}";

  // Shared helpers, then the field of the chosen preset, then the character pass.
  var HEAD = [
    "precision highp float;",
    "uniform vec2 u_res;uniform float u_time;uniform sampler2D u_atlas;",
    "uniform float u_count;uniform vec2 u_cell;uniform vec2 u_atlasCell;",
    "uniform vec3 u_bg;uniform vec3 u_fg;uniform vec3 u_dim;",
    "uniform float u_gain;uniform float u_floor;",
    "float hash(vec2 p){return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453);}",
    "float noise(vec2 p){vec2 i=floor(p),f=fract(p);f=f*f*(3.-2.*f);",
    " return mix(mix(hash(i),hash(i+vec2(1,0)),f.x),",
    "            mix(hash(i+vec2(0,1)),hash(i+vec2(1,1)),f.x),f.y);}",
    "float fbm(vec2 p){float v=0.,a=.5;mat2 m=mat2(1.6,1.2,-1.2,1.6);",
    " for(int i=0;i<5;i++){v+=a*noise(p);p=m*p;a*=.5;}return v;}"
  ].join("\n");

  // Each field returns 0 to 1: how much ink this point deserves.
  var FIELDS = {
    // Slow domain-warped smoke.
    drift:
      "float field(vec2 uv){vec2 p=uv*vec2(u_res.x/u_res.y,1.)*2.6;float t=u_time*.06;" +
      "vec2 q=vec2(fbm(p+t),fbm(p+vec2(5.2,1.3)-t*.8));" +
      "vec2 r=vec2(fbm(p+2.4*q+vec2(1.7,9.2)+t*.5),fbm(p+2.4*q+vec2(8.3,2.8)-t*.4));" +
      "float f=fbm(p+2.2*r);return smoothstep(.28,.82,f);}",

    // Standing sound waves crossing the page.
    wave:
      "float field(vec2 uv){float t=u_time*.35;float y=uv.y-.5;" +
      "float w=sin(uv.x*7.+t)*.13+sin(uv.x*13.-t*1.4)*.07+sin(uv.x*3.-t*.6)*.05;" +
      "float d=abs(y-w);float band=smoothstep(.30,.0,d);" +
      "float g=fbm(vec2(uv.x*4.-t*.2,uv.y*4.))*.35;" +
      "return clamp(band*.85+g,0.,1.);}",

    // A glow rising from behind the window frame, the way the god rays sat.
    bloom:
      "float field(vec2 uv){vec2 p=vec2((uv.x-.5)*(u_res.x/u_res.y),uv.y-1.15);" +
      "float d=length(p*vec2(.85,1.));float core=smoothstep(1.15,.08,d);" +
      "float t=u_time*.1;float g=fbm(vec2(uv.x*3.4,uv.y*3.4)+vec2(t,-t*.7));" +
      "return clamp(core*(.55+.7*g),0.,1.);}",

    // Wide bands folding through each other.
    fold:
      "float field(vec2 uv){vec2 p=uv*vec2(u_res.x/u_res.y,1.)*1.8;float t=u_time*.09;" +
      "float a=fbm(p+vec2(t,0.));float b=fbm(p*1.7+vec2(-t*.7,t*.4)+a*1.6);" +
      "float s=sin((p.x+p.y)*2.2+b*4.2+t*1.6)*.5+.5;" +
      "return smoothstep(.18,.92,s*.65+b*.5);}"
  };

  var BODY = [
    "void main(){",
    " vec2 cell=u_cell;",
    " vec2 g=floor(gl_FragCoord.xy/cell);",
    " vec2 f=fract(gl_FragCoord.xy/cell);",
    " vec2 c=(g*cell+cell*.5)/u_res;",
    " float L=clamp(field(c)*u_gain+u_floor,0.,1.);",
    " float idx=floor(L*(u_count-.001));",
    " vec2 auv=vec2((idx+f.x)/u_count,1.-f.y);",
    " float ink=texture2D(u_atlas,auv).a;",
    " vec3 lit=mix(u_dim,u_fg,smoothstep(.15,.95,L));",
    " gl_FragColor=vec4(mix(u_bg,lit,ink),1.);",
    "}"
  ].join("\n");

  function compile(gl, type, src) {
    var s = gl.createShader(type);
    gl.shaderSource(s, src);
    gl.compileShader(s);
    if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
      throw new Error(gl.getShaderInfoLog(s) || "shader failed");
    }
    return s;
  }

  /* Bake the character set into one strip of glyphs, in the page's own face. */
  function atlas(chars, cw, ch, font) {
    var cv = document.createElement("canvas");
    cv.width = chars.length * cw;
    cv.height = ch;
    var x = cv.getContext("2d");
    x.clearRect(0, 0, cv.width, cv.height);
    x.font = "700 " + Math.round(ch * 0.82) + "px " + font;
    x.fillStyle = "#fff";
    x.textAlign = "center";
    x.textBaseline = "middle";
    for (var i = 0; i < chars.length; i++) {
      x.fillText(chars.charAt(i), i * cw + cw / 2, ch / 2 + ch * 0.04);
    }
    return cv;
  }

  function rgb(hex) {
    var h = hex.replace("#", "");
    if (h.length === 3) h = h[0] + h[0] + h[1] + h[1] + h[2] + h[2];
    var n = parseInt(h, 16);
    return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
  }

  return function mount(host, preset) {
    var canvas = document.createElement("canvas");
    canvas.setAttribute("aria-hidden", "true");
    host.appendChild(canvas);
    var gl = canvas.getContext("webgl", { antialias: false, alpha: false });
    if (!gl) return null;

    var field = FIELDS[preset.field] || FIELDS.drift;
    var prog = gl.createProgram();
    try {
      gl.attachShader(prog, compile(gl, gl.VERTEX_SHADER, VERT));
      gl.attachShader(prog, compile(gl, gl.FRAGMENT_SHADER,
        HEAD + "\n" + field + "\n" + BODY));
      gl.linkProgram(prog);
      if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
        throw new Error(gl.getProgramInfoLog(prog) || "link failed");
      }
    } catch (e) {
      host.removeChild(canvas);
      return null;
    }
    gl.useProgram(prog);

    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
    var loc = gl.getAttribLocation(prog, "a");
    gl.enableVertexAttribArray(loc);
    gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);

    var AW = 26, AH = 44;                       // one glyph in the atlas
    var tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE,
      atlas(preset.chars, AW, AH, preset.font ||
        'Consolas,"Cascadia Mono",ui-monospace,monospace'));
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.uniform1i(gl.getUniformLocation(prog, "u_atlas"), 0);

    var U = {};
    ["u_res", "u_time", "u_count", "u_cell", "u_bg", "u_fg", "u_dim",
     "u_gain", "u_floor"].forEach(function (n) {
      U[n] = gl.getUniformLocation(prog, n);
    });
    gl.uniform1f(U.u_count, preset.chars.length);

    var theme = "dark", speed = preset.speed === undefined ? 1 : preset.speed;
    var t0 = performance.now(), clock = 0, raf = null, running = false;

    function colors() {
      var c = preset[theme] || preset.dark;
      gl.uniform3fv(U.u_bg, rgb(c.bg));
      gl.uniform3fv(U.u_fg, rgb(c.fg));
      gl.uniform3fv(U.u_dim, rgb(c.dim));
    }

    function size() {
      var dpr = Math.min(window.devicePixelRatio || 1, 1.5);
      var w = Math.max(1, Math.round(host.clientWidth * dpr));
      var h = Math.max(1, Math.round(host.clientHeight * dpr));
      if (canvas.width !== w || canvas.height !== h) {
        canvas.width = w;
        canvas.height = h;
      }
      gl.viewport(0, 0, w, h);
      gl.uniform2f(U.u_res, w, h);
      var ch = preset.cell * dpr;
      gl.uniform2f(U.u_cell, ch * (AW / AH), ch);
    }

    function draw() {
      size();
      colors();
      gl.uniform1f(U.u_gain, preset.gain === undefined ? 1 : preset.gain);
      gl.uniform1f(U.u_floor, preset.floor === undefined ? 0 : preset.floor);
      gl.uniform1f(U.u_time, clock);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
    }

    function loop(now) {
      clock += ((now - t0) / 1000) * speed;
      t0 = now;
      draw();
      raf = requestAnimationFrame(loop);
    }

    function start() {
      if (running) return;
      running = true;
      t0 = performance.now();
      raf = requestAnimationFrame(loop);
    }

    function stop() {
      running = false;
      if (raf) cancelAnimationFrame(raf);
      raf = null;
    }

    window.addEventListener("resize", function () { if (!running) draw(); });
    draw();

    return {
      canvas: canvas,
      start: start,
      stop: stop,
      still: function (at) { clock = at === undefined ? 12 : at; draw(); },
      setTheme: function (t) { theme = t; draw(); },
      setSpeed: function (s) { speed = s; }
    };
  };
})();
