interface SkeletonProps {
  className?: string
}

export function Skeleton({ className = '' }: SkeletonProps) {
  return (
    <div className={`bg-white/5 rounded-lg animate-pulse ${className}`} />
  )
}