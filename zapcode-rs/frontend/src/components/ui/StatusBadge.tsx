type Status = 'confirmed' | 'pending' | 'failed'

export function StatusBadge({ status }: { status: Status }) {
  const map: Record<Status, string> = {
    confirmed: 'text-ok border-ok/20 bg-ok/8',
    pending:   'text-gold border-gold/20 bg-gold/8',
    failed:    'text-danger border-danger/20 bg-danger/8',
  }
  return (
    <span className={`text-[9px] tracking-widest uppercase px-2 py-0.5 rounded-full border font-bold ${map[status]}`}>
      {status}
    </span>
  )
}