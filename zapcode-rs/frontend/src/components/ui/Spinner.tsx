interface SpinnerProps {
  size?: number
}

export function Spinner({ size = 16 }: SpinnerProps) {
  return (
    <span
      className="inline-block rounded-full border-2 border-white/10 border-t-gold animate-spin flex-shrink-0"
      style={{ width: size, height: size }}
    />
  )
}