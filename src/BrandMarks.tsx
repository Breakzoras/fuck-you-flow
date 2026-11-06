import claudeSpark from "./assets/brands/claude-spark-clay.svg";
import openaiDark from "./assets/brands/openai-blossom-white.svg";
import openaiLight from "./assets/brands/openai-blossom-black.svg";

/** The Claude and OpenAI marks beside the "dictate through Claude and Codex"
 *  setting and in the What's new window. The files are the companies' own,
 *  unchanged: the Claude Spark from Anthropic's press kit and the OpenAI
 *  Blossom from openai.com/brand (downloaded 6 November 2026). Claude is a
 *  trademark of Anthropic, OpenAI, ChatGPT and Codex of OpenAI; FU Flow is not
 *  affiliated with either. The Blossom comes in white and black, and the theme
 *  picks one through CSS (.brand-on-dark / .brand-on-light in App.css). */
export default function BrandMarks({ size = 24, gap = 8 }: { size?: number; gap?: number }) {
  return (
    <span className="brand-marks" style={{ gap: gap + Math.round(size * 0.45) }} aria-hidden="true">
      <img src={claudeSpark} width={size} height={size} alt="" />
      <img className="oa brand-on-dark" src={openaiDark} width={size} height={size} alt="" />
      <img className="oa brand-on-light" src={openaiLight} width={size} height={size} alt="" />
    </span>
  );
}
