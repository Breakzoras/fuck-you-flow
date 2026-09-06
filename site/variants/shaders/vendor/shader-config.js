// Generated from @paper-design/shaders-react 0.0.80 (Apache-2.0).
// Each entry: the fragment shader, its default params and the exact
// prop -> uniform mapping the official React component performs.
var PS = window.PaperShaders;
// The transcribed blocks call the library helpers by their bare names.
for (var __k in PS) { if (!(__k in window)) window[__k] = PS[__k]; }
var SHADERS = {};

SHADERS["godRays"] = {
  frag: PS.godRaysFragmentShader,
  defaults: Object.assign({"fit": "contain", "scale": 1, "rotation": 0, "offsetX": 0, "offsetY": 0, "originX": 0.5, "originY": 0.5, "worldWidth": 0, "worldHeight": 0, }, {
    
    offsetX: 0,
    offsetY: -0.55,
    colorBack: "#000000",
    colorBloom: "#0000ff",
    colors: ["#a600ff6e", "#6200fff0", "#ffffff", "#33fff5"],
    density: 0.3,
    spotty: 0.3,
    midIntensity: 0.4,
    midSize: 0.2,
    intensity: 0.8,
    bloom: 0.4,
    speed: 0.75,
    frame: 0
  }),
  build: function (p) {
    var speed = p.speed, frame = p.frame, colorBloom = p.colorBloom, colorBack = p.colorBack, colors = p.colors, density = p.density, spotty = p.spotty, midIntensity = p.midIntensity, midSize = p.midSize, intensity = p.intensity, bloom = p.bloom, fit = p.fit, scale = p.scale, rotation = p.rotation, originX = p.originX, originY = p.originY, offsetX = p.offsetX, offsetY = p.offsetY, worldWidth = p.worldWidth, worldHeight = p.worldHeight;
    var processedImage = p.image || PS.emptyPixel;
    var uniforms = {
    // Own uniforms
    u_colorBloom: getShaderColorFromString(colorBloom),
    u_colorBack: getShaderColorFromString(colorBack),
    u_colors: colors.map(getShaderColorFromString),
    u_colorsCount: colors.length,
    u_density: density,
    u_spotty: spotty,
    u_midIntensity: midIntensity,
    u_midSize: midSize,
    u_intensity: intensity,
    u_bloom: bloom,
    u_noiseTexture: getShaderNoiseTexture(),
    // Sizing uniforms
    u_fit: ShaderFitOptions[fit],
    u_scale: scale,
    u_rotation: rotation,
    u_offsetX: offsetX,
    u_offsetY: offsetY,
    u_originX: originX,
    u_originY: originY,
    u_worldWidth: worldWidth,
    u_worldHeight: worldHeight
  };
    return uniforms;
  }
};

SHADERS["meshGradient"] = {
  frag: PS.meshGradientFragmentShader,
  defaults: Object.assign({"fit": "contain", "scale": 1, "rotation": 0, "offsetX": 0, "offsetY": 0, "originX": 0.5, "originY": 0.5, "worldWidth": 0, "worldHeight": 0, }, {
    
    speed: 1,
    frame: 0,
    colors: ["#e0eaff", "#241d9a", "#f75092", "#9f50d3"],
    distortion: 0.8,
    swirl: 0.1,
    grainMixer: 0,
    grainOverlay: 0
  }),
  build: function (p) {
    var speed = p.speed, frame = p.frame, colors = p.colors, distortion = p.distortion, swirl = p.swirl, grainMixer = p.grainMixer, grainOverlay = p.grainOverlay, fit = p.fit, rotation = p.rotation, scale = p.scale, originX = p.originX, originY = p.originY, offsetX = p.offsetX, offsetY = p.offsetY, worldWidth = p.worldWidth, worldHeight = p.worldHeight;
    var processedImage = p.image || PS.emptyPixel;
    var uniforms = {
    // Own uniforms
    u_colors: colors.map(getShaderColorFromString),
    u_colorsCount: colors.length,
    u_distortion: distortion,
    u_swirl: swirl,
    u_grainMixer: grainMixer,
    u_grainOverlay: grainOverlay,
    // Sizing uniforms
    u_fit: ShaderFitOptions[fit],
    u_rotation: rotation,
    u_scale: scale,
    u_offsetX: offsetX,
    u_offsetY: offsetY,
    u_originX: originX,
    u_originY: originY,
    u_worldWidth: worldWidth,
    u_worldHeight: worldHeight
  };
    return uniforms;
  }
};

SHADERS["grainGradient"] = {
  frag: PS.grainGradientFragmentShader,
  defaults: Object.assign({"fit": "contain", "scale": 1, "rotation": 0, "offsetX": 0, "offsetY": 0, "originX": 0.5, "originY": 0.5, "worldWidth": 0, "worldHeight": 0, }, {
    
    speed: 1,
    frame: 0,
    colorBack: "#000000",
    colors: ["#7300ff", "#eba8ff", "#00bfff", "#2a00ff"],
    softness: 0.5,
    intensity: 0.5,
    noise: 0.25,
    shape: "corners"
  }),
  build: function (p) {
    var speed = p.speed, frame = p.frame, colorBack = p.colorBack, colors = p.colors, softness = p.softness, intensity = p.intensity, noise = p.noise, shape = p.shape, fit = p.fit, scale = p.scale, rotation = p.rotation, originX = p.originX, originY = p.originY, offsetX = p.offsetX, offsetY = p.offsetY, worldWidth = p.worldWidth, worldHeight = p.worldHeight;
    var processedImage = p.image || PS.emptyPixel;
    var uniforms = {
    // Own uniforms
    u_colorBack: getShaderColorFromString(colorBack),
    u_colors: colors.map(getShaderColorFromString),
    u_colorsCount: colors.length,
    u_softness: softness,
    u_intensity: intensity,
    u_noise: noise,
    u_shape: GrainGradientShapes[shape],
    u_noiseTexture: getShaderNoiseTexture(),
    // Sizing uniforms
    u_fit: ShaderFitOptions[fit],
    u_scale: scale,
    u_rotation: rotation,
    u_offsetX: offsetX,
    u_offsetY: offsetY,
    u_originX: originX,
    u_originY: originY,
    u_worldWidth: worldWidth,
    u_worldHeight: worldHeight
  };
    return uniforms;
  }
};

SHADERS["smokeRing"] = {
  frag: PS.smokeRingFragmentShader,
  defaults: Object.assign({"fit": "contain", "scale": 1, "rotation": 0, "offsetX": 0, "offsetY": 0, "originX": 0.5, "originY": 0.5, "worldWidth": 0, "worldHeight": 0, }, {
    
    speed: 0.5,
    frame: 0,
    colorBack: "#000000",
    colors: ["#ffffff"],
    noiseScale: 3,
    noiseIterations: 8,
    radius: 0.25,
    thickness: 0.65,
    innerShape: 0.7,
    scale: 0.8
  }),
  build: function (p) {
    var speed = p.speed, frame = p.frame, colorBack = p.colorBack, colors = p.colors, noiseScale = p.noiseScale, thickness = p.thickness, radius = p.radius, innerShape = p.innerShape, noiseIterations = p.noiseIterations, fit = p.fit, scale = p.scale, rotation = p.rotation, originX = p.originX, originY = p.originY, offsetX = p.offsetX, offsetY = p.offsetY, worldWidth = p.worldWidth, worldHeight = p.worldHeight;
    var processedImage = p.image || PS.emptyPixel;
    var uniforms = {
    // Own uniforms
    u_colorBack: getShaderColorFromString(colorBack),
    u_colors: colors.map(getShaderColorFromString),
    u_colorsCount: colors.length,
    u_noiseScale: noiseScale,
    u_thickness: thickness,
    u_radius: radius,
    u_innerShape: innerShape,
    u_noiseIterations: noiseIterations,
    u_noiseTexture: getShaderNoiseTexture(),
    // Sizing uniforms
    u_fit: ShaderFitOptions[fit],
    u_scale: scale,
    u_rotation: rotation,
    u_offsetX: offsetX,
    u_offsetY: offsetY,
    u_originX: originX,
    u_originY: originY,
    u_worldWidth: worldWidth,
    u_worldHeight: worldHeight
  };
    return uniforms;
  }
};

SHADERS["liquidMetal"] = {
  frag: PS.liquidMetalFragmentShader,
  defaults: Object.assign({"fit": "contain", "scale": 1, "rotation": 0, "offsetX": 0, "offsetY": 0, "originX": 0.5, "originY": 0.5, "worldWidth": 0, "worldHeight": 0, }, {
    
    scale: 0.6,
    speed: 1,
    frame: 0,
    colorBack: "#AAAAAC",
    colorTint: "#ffffff",
    distortion: 0.07,
    repetition: 2,
    shiftRed: 0.3,
    shiftBlue: 0.3,
    contour: 0.4,
    softness: 0.1,
    angle: 70,
    shape: "diamond"
  }),
  build: function (p) {
    var colorBack = p.colorBack, colorTint = p.colorTint, speed = p.speed, frame = p.frame, image = p.image, contour = p.contour, distortion = p.distortion, softness = p.softness, repetition = p.repetition, shiftRed = p.shiftRed, shiftBlue = p.shiftBlue, angle = p.angle, shape = p.shape, suspendWhenProcessingImage = p.suspendWhenProcessingImage, fit = p.fit, scale = p.scale, rotation = p.rotation, originX = p.originX, originY = p.originY, offsetX = p.offsetX, offsetY = p.offsetY, worldWidth = p.worldWidth, worldHeight = p.worldHeight;
    var processedImage = p.image || PS.emptyPixel;
    var uniforms = {
    // Own uniforms
    u_colorBack: getShaderColorFromString(colorBack),
    u_colorTint: getShaderColorFromString(colorTint),
    u_image: processedImage,
    u_contour: contour,
    u_distortion: distortion,
    u_softness: softness,
    u_repetition: repetition,
    u_shiftRed: shiftRed,
    u_shiftBlue: shiftBlue,
    u_angle: angle,
    u_isImage: Boolean(image),
    u_shape: LiquidMetalShapes[shape],
    // Sizing uniforms
    u_fit: ShaderFitOptions[fit],
    u_scale: scale,
    u_rotation: rotation,
    u_offsetX: offsetX,
    u_offsetY: offsetY,
    u_originX: originX,
    u_originY: originY,
    u_worldWidth: worldWidth,
    u_worldHeight: worldHeight
  };
    return uniforms;
  }
};

SHADERS["neuroNoise"] = {
  frag: PS.neuroNoiseFragmentShader,
  defaults: Object.assign({"fit": "none", "scale": 1, "rotation": 0, "offsetX": 0, "offsetY": 0, "originX": 0.5, "originY": 0.5, "worldWidth": 0, "worldHeight": 0, }, {
    
    speed: 1,
    frame: 0,
    colorFront: "#ffffff",
    colorMid: "#47a6ff",
    colorBack: "#000000",
    brightness: 0.05,
    contrast: 0.3
  }),
  build: function (p) {
    var speed = p.speed, frame = p.frame, colorFront = p.colorFront, colorMid = p.colorMid, colorBack = p.colorBack, brightness = p.brightness, contrast = p.contrast, fit = p.fit, scale = p.scale, rotation = p.rotation, originX = p.originX, originY = p.originY, offsetX = p.offsetX, offsetY = p.offsetY, worldWidth = p.worldWidth, worldHeight = p.worldHeight;
    var processedImage = p.image || PS.emptyPixel;
    var uniforms = {
    // Own uniforms
    u_colorFront: getShaderColorFromString(colorFront),
    u_colorMid: getShaderColorFromString(colorMid),
    u_colorBack: getShaderColorFromString(colorBack),
    u_brightness: brightness,
    u_contrast: contrast,
    // Sizing uniforms
    u_fit: ShaderFitOptions[fit],
    u_scale: scale,
    u_rotation: rotation,
    u_offsetX: offsetX,
    u_offsetY: offsetY,
    u_originX: originX,
    u_originY: originY,
    u_worldWidth: worldWidth,
    u_worldHeight: worldHeight
  };
    return uniforms;
  }
};

SHADERS["warp"] = {
  frag: PS.warpFragmentShader,
  defaults: Object.assign({"fit": "none", "scale": 1, "rotation": 0, "offsetX": 0, "offsetY": 0, "originX": 0.5, "originY": 0.5, "worldWidth": 0, "worldHeight": 0, }, {
    
    rotation: 0,
    speed: 1,
    frame: 0,
    colors: ["#121212", "#9470ff", "#121212", "#8838ff"],
    proportion: 0.45,
    softness: 1,
    distortion: 0.25,
    swirl: 0.8,
    swirlIterations: 10,
    shapeScale: 0.1,
    shape: "checks"
  }),
  build: function (p) {
    var speed = p.speed, frame = p.frame, colors = p.colors, proportion = p.proportion, softness = p.softness, distortion = p.distortion, swirl = p.swirl, swirlIterations = p.swirlIterations, shapeScale = p.shapeScale, shape = p.shape, fit = p.fit, scale = p.scale, rotation = p.rotation, originX = p.originX, originY = p.originY, offsetX = p.offsetX, offsetY = p.offsetY, worldWidth = p.worldWidth, worldHeight = p.worldHeight;
    var processedImage = p.image || PS.emptyPixel;
    var uniforms = {
    // Own uniforms
    u_colors: colors.map(getShaderColorFromString),
    u_colorsCount: colors.length,
    u_proportion: proportion,
    u_softness: softness,
    u_distortion: distortion,
    u_swirl: swirl,
    u_swirlIterations: swirlIterations,
    u_shapeScale: shapeScale,
    u_shape: WarpPatterns[shape],
    u_noiseTexture: getShaderNoiseTexture(),
    // Sizing uniforms
    u_scale: scale,
    u_rotation: rotation,
    u_fit: ShaderFitOptions[fit],
    u_offsetX: offsetX,
    u_offsetY: offsetY,
    u_originX: originX,
    u_originY: originY,
    u_worldWidth: worldWidth,
    u_worldHeight: worldHeight
  };
    return uniforms;
  }
};

SHADERS["dithering"] = {
  frag: PS.ditheringFragmentShader,
  defaults: Object.assign({"fit": "none", "scale": 1, "rotation": 0, "offsetX": 0, "offsetY": 0, "originX": 0.5, "originY": 0.5, "worldWidth": 0, "worldHeight": 0, }, {
    
    speed: 1,
    frame: 0,
    scale: 0.6,
    colorBack: "#000000",
    colorFront: "#00b2ff",
    shape: "sphere",
    type: "4x4",
    size: 2
  }),
  build: function (p) {
    var speed = p.speed, frame = p.frame, colorBack = p.colorBack, colorFront = p.colorFront, shape = p.shape, type = p.type, size = p.size, fit = p.fit, scale = p.scale, rotation = p.rotation, originX = p.originX, originY = p.originY, offsetX = p.offsetX, offsetY = p.offsetY, worldWidth = p.worldWidth, worldHeight = p.worldHeight;
    var processedImage = p.image || PS.emptyPixel;
    var uniforms = {
    // Own uniforms
    u_colorBack: getShaderColorFromString(colorBack),
    u_colorFront: getShaderColorFromString(colorFront),
    u_shape: DitheringShapes[shape],
    u_type: DitheringTypes[type],
    u_pxSize: size,
    // Sizing uniforms
    u_fit: ShaderFitOptions[fit],
    u_scale: scale,
    u_rotation: rotation,
    u_offsetX: offsetX,
    u_offsetY: offsetY,
    u_originX: originX,
    u_originY: originY,
    u_worldWidth: worldWidth,
    u_worldHeight: worldHeight
  };
    return uniforms;
  }
};

SHADERS["waves"] = {
  frag: PS.wavesFragmentShader,
  defaults: Object.assign({"fit": "none", "scale": 1, "rotation": 0, "offsetX": 0, "offsetY": 0, "originX": 0.5, "originY": 0.5, "worldWidth": 0, "worldHeight": 0, }, {
    
    scale: 0.6,
    colorFront: "#ffbb00",
    colorBack: "#000000",
    shape: 0,
    frequency: 0.5,
    amplitude: 0.5,
    spacing: 1.2,
    proportion: 0.1,
    softness: 0
  }),
  build: function (p) {
    var colorFront = p.colorFront, colorBack = p.colorBack, shape = p.shape, frequency = p.frequency, amplitude = p.amplitude, spacing = p.spacing, proportion = p.proportion, softness = p.softness, fit = p.fit, scale = p.scale, rotation = p.rotation, offsetX = p.offsetX, offsetY = p.offsetY, originX = p.originX, originY = p.originY, worldWidth = p.worldWidth, worldHeight = p.worldHeight, maxPixelCount = p.maxPixelCount;
    var processedImage = p.image || PS.emptyPixel;
    var uniforms = {
    // Own uniforms
    u_colorFront: getShaderColorFromString(colorFront),
    u_colorBack: getShaderColorFromString(colorBack),
    u_shape: shape,
    u_frequency: frequency,
    u_amplitude: amplitude,
    u_spacing: spacing,
    u_proportion: proportion,
    u_softness: softness,
    // Sizing uniforms
    u_fit: ShaderFitOptions[fit],
    u_scale: scale,
    u_rotation: rotation,
    u_offsetX: offsetX,
    u_offsetY: offsetY,
    u_originX: originX,
    u_originY: originY,
    u_worldWidth: worldWidth,
    u_worldHeight: worldHeight
  };
    return uniforms;
  }
};

SHADERS["gemSmoke"] = {
  frag: PS.gemSmokeFragmentShader,
  defaults: Object.assign({"fit": "contain", "scale": 1, "rotation": 0, "offsetX": 0, "offsetY": 0, "originX": 0.5, "originY": 0.5, "worldWidth": 0, "worldHeight": 0, }, {
    
    scale: 0.6,
    speed: 1,
    frame: 0,
    colorBack: "#f0efea",
    colorInner: "#fafaf5",
    colors: ["#333333", "#e7e6df"],
    outerGlow: 0.55,
    innerGlow: 1,
    innerDistortion: 0.8,
    outerDistortion: 0.6,
    offset: 0,
    angle: 0,
    size: 0.8,
    shape: "diamond"
  }),
  build: function (p) {
    var colorBack = p.colorBack, colors = p.colors, speed = p.speed, frame = p.frame, image = p.image, innerDistortion = p.innerDistortion, outerDistortion = p.outerDistortion, outerGlow = p.outerGlow, innerGlow = p.innerGlow, colorInner = p.colorInner, offset = p.offset, angle = p.angle, size = p.size, shape = p.shape, suspendWhenProcessingImage = p.suspendWhenProcessingImage, fit = p.fit, scale = p.scale, rotation = p.rotation, originX = p.originX, originY = p.originY, offsetX = p.offsetX, offsetY = p.offsetY, worldWidth = p.worldWidth, worldHeight = p.worldHeight;
    var processedImage = p.image || PS.emptyPixel;
    var uniforms = {
    // Own uniforms
    u_colors: colors.map(getShaderColorFromString),
    u_colorsCount: colors.length,
    u_colorBack: getShaderColorFromString(colorBack),
    u_image: processedImage,
    u_innerDistortion: innerDistortion,
    u_outerDistortion: outerDistortion,
    u_outerGlow: outerGlow,
    u_innerGlow: innerGlow,
    u_colorInner: getShaderColorFromString(colorInner),
    u_offset: offset,
    u_angle: angle,
    u_size: size,
    u_isImage: Boolean(image),
    u_shape: GemSmokeShapes[shape],
    // Sizing uniforms
    u_fit: ShaderFitOptions[fit],
    u_scale: scale,
    u_rotation: rotation,
    u_offsetX: offsetX,
    u_offsetY: offsetY,
    u_originX: originX,
    u_originY: originY,
    u_worldWidth: worldWidth,
    u_worldHeight: worldHeight
  };
    return uniforms;
  }
};