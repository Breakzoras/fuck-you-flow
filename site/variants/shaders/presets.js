/* Fuck You Flow: the ten shader candidates, tuned to the brand.
   Black, one acid accent (#c8ff5a), deeper green (#8fd400) for depth.
   Every entry carries a dark and a light set, because the site has both themes.
   "dim" is the canvas opacity, the readability control: the copy sits on top.
   Base parameters and their names come from @paper-design/shaders (Apache-2.0). */
var DEMOS = {

  godRays: {
    title: "God rays",
    note: "Beams of light rising from behind the window frame. The most dramatic of the ten.",
    dark: {
      colorBack: "#000000", colorBloom: "#12210a",
      colors: ["#c8ff5a55", "#8fd400cc", "#f2f2f2", "#c8ff5a"],
      density: 0.26, spotty: 0.22, midSize: 0.22, midIntensity: 0.35,
      intensity: 0.72, bloom: 0.32, offsetY: -0.62, scale: 1, speed: 0.45, dim: 0.85
    },
    light: {
      colorBack: "#f6f7f2", colorBloom: "#e8f2d8",
      colors: ["#8fd40033", "#c8ff5a88", "#ffffff", "#6fae00"],
      density: 0.24, spotty: 0.18, midSize: 0.24, midIntensity: 0.3,
      intensity: 0.5, bloom: 0.5, offsetY: -0.62, scale: 1, speed: 0.45, dim: 0.75
    }
  },

  meshGradient: {
    title: "Mesh gradient",
    note: "Soft fields of colour drifting into each other. The quiet, expensive default.",
    dark: {
      colors: ["#000000", "#16240a", "#c8ff5a", "#3f6600"],
      distortion: 0.85, swirl: 0.14, grainMixer: 0.18, grainOverlay: 0.05,
      scale: 1.1, speed: 0.45, dim: 0.9
    },
    light: {
      colors: ["#f6f7f2", "#e4efd0", "#c8ff5a", "#a9d76a"],
      distortion: 0.85, swirl: 0.14, grainMixer: 0.15, grainOverlay: 0.04,
      scale: 1.1, speed: 0.45, dim: 0.85
    }
  },

  grainGradient: {
    title: "Grain gradient",
    note: "The same soft field with film grain over it. Reads like printed matter.",
    dark: {
      colorBack: "#000000",
      colors: ["#c8ff5a", "#3f6600", "#8fd400", "#0b1403"],
      softness: 0.6, intensity: 0.4, noise: 0.32, shape: "corners",
      scale: 1, speed: 0.5, dim: 0.9
    },
    light: {
      colorBack: "#f6f7f2",
      colors: ["#8fd400", "#c8ff5a", "#dcecc0", "#6fae00"],
      softness: 0.6, intensity: 0.35, noise: 0.22, shape: "corners",
      scale: 1, speed: 0.5, dim: 0.8
    }
  },

  smokeRing: {
    title: "Smoke ring",
    note: "A ring of smoke breathing behind the copy. Atmospheric and slow.",
    dark: {
      colorBack: "#000000", colors: ["#c8ff5a"],
      noiseScale: 2.4, noiseIterations: 8, radius: 0.3, thickness: 0.6,
      innerShape: 0.72, scale: 0.9, speed: 0.32, dim: 0.4
    },
    light: {
      colorBack: "#f6f7f2", colors: ["#6fae00"],
      noiseScale: 2.4, noiseIterations: 8, radius: 0.3, thickness: 0.6,
      innerShape: 0.72, scale: 0.9, speed: 0.32, dim: 0.4
    }
  },

  liquidMetal: {
    title: "Liquid metal",
    note: "A poured metal surface with a green tint. The most product-like of the ten.",
    dark: {
      colorBack: "#000000", colorTint: "#c8ff5a", shape: "metaballs",
      contour: 0.5, distortion: 0.1, softness: 0.2, repetition: 2.8,
      shiftRed: 0.02, shiftBlue: 0.04, angle: 70, scale: 2.4, speed: 0.6, dim: 0.5
    },
    light: {
      colorBack: "#f6f7f2", colorTint: "#8fd400", shape: "metaballs",
      contour: 0.45, distortion: 0.1, softness: 0.22, repetition: 2.8,
      shiftRed: 0.02, shiftBlue: 0.04, angle: 70, scale: 2.4, speed: 0.6, dim: 0.45
    }
  },

  neuroNoise: {
    title: "Neuro noise",
    note: "Fine filaments that keep rewiring themselves. Technical without being busy.",
    dark: {
      colorFront: "#c8ff5a", colorMid: "#2f4a08", colorBack: "#000000",
      brightness: 0.04, contrast: 0.34, scale: 0.8, speed: 0.5, dim: 0.85
    },
    light: {
      colorFront: "#4d7a00", colorMid: "#b7dc85", colorBack: "#f6f7f2",
      brightness: 0.06, contrast: 0.28, scale: 0.8, speed: 0.5, dim: 0.6
    }
  },

  warp: {
    title: "Warp",
    note: "Wide bands bending through each other. Motion you read as depth.",
    dark: {
      colors: ["#000000", "#243a06", "#000000", "#c8ff5a"],
      proportion: 0.45, softness: 1, distortion: 0.28, swirl: 0.85,
      swirlIterations: 10, shapeScale: 0.09, shape: "checks",
      scale: 1.1, speed: 0.4, dim: 0.85
    },
    light: {
      colors: ["#f6f7f2", "#dbe9c2", "#f6f7f2", "#8fd400"],
      proportion: 0.45, softness: 1, distortion: 0.28, swirl: 0.85,
      swirlIterations: 10, shapeScale: 0.09, shape: "checks",
      scale: 1.1, speed: 0.4, dim: 0.8
    }
  },

  dithering: {
    title: "Dithering",
    note: "An ordered dot screen, the way an old monitor draws a gradient. Closest to the app itself.",
    dark: {
      colorBack: "#000000", colorFront: "#c8ff5a",
      shape: "warp", type: "4x4", size: 2, scale: 0.5, speed: 0.4, dim: 0.3
    },
    light: {
      colorBack: "#f6f7f2", colorFront: "#4d7a00",
      shape: "warp", type: "4x4", size: 2, scale: 0.5, speed: 0.4, dim: 0.35
    }
  },

  waves: {
    title: "Waves",
    note: "Standing sound waves. The literal reading of what the app does.",
    dark: {
      colorFront: "#c8ff5a", colorBack: "#000000",
      shape: 0, frequency: 0.28, amplitude: 0.55, spacing: 2.6,
      proportion: 0.1, softness: 0.45, scale: 0.45, dim: 0.4
    },
    light: {
      colorFront: "#4d7a00", colorBack: "#f6f7f2",
      shape: 0, frequency: 0.4, amplitude: 0.5, spacing: 1.4,
      proportion: 0.1, softness: 0.3, scale: 0.55, dim: 0.45
    }
  },

  gemSmoke: {
    title: "Gem smoke",
    note: "A cut stone glowing through smoke. The richest, and the heaviest.",
    dark: {
      colorBack: "#000000", colorInner: "#0e1703",
      colors: ["#c8ff5a", "#2f4a08"],
      outerGlow: 0.6, innerGlow: 1, innerDistortion: 0.8, outerDistortion: 0.6,
      offset: 0, angle: 0, size: 0.8, shape: "diamond",
      scale: 0.55, speed: 0.5, dim: 0.5
    },
    light: {
      colorBack: "#f6f7f2", colorInner: "#ffffff",
      colors: ["#8fd400", "#dfe9cc"],
      outerGlow: 0.55, innerGlow: 1, innerDistortion: 0.8, outerDistortion: 0.6,
      offset: 0, angle: 0, size: 0.8, shape: "diamond",
      scale: 0.55, speed: 0.5, dim: 0.8
    }
  }
};

var DEMO_ORDER = ["godRays", "meshGradient", "grainGradient", "smokeRing", "liquidMetal",
                  "neuroNoise", "warp", "dithering", "waves", "gemSmoke"];
