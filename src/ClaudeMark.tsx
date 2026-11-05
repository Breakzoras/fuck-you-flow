/** The Claude mark: a coral spark of rounded rays. Marks the setting that
 *  hands dictation to the Claude app, so the user can find it at a glance. */
export default function ClaudeMark({ size = 24 }: { size?: number }) {
  const rays = 12;
  return (
    <svg width={size} height={size} viewBox="-50 -50 100 100" role="img" aria-label="Claude" style={{ flexShrink: 0 }}>
      <circle r="9" fill="#D97757" />
      {Array.from({ length: rays }, (_, i) => {
        const long = i % 2 === 0;
        return (
          <rect
            key={i}
            x={-4.5}
            y={long ? -46 : -38}
            width={9}
            height={long ? 40 : 32}
            rx={4.5}
            fill="#D97757"
            transform={`rotate(${(360 / rays) * i + (i % 3) * 4})`}
          />
        );
      })}
    </svg>
  );
}
