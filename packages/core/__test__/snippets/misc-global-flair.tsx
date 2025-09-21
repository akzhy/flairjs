import { flair } from '@flairjs/client'
import clsx from 'clsx'

export const Box = ({ children, containerClassName }: { children?: React.ReactNode; containerClassName?: string }) => {
  return (
    <div className={clsx('box', containerClassName)}>
      {children}
      <span className="title">Hello</span>
    </div>
  )
}

Box.flair = flair({
  '.box': { color: 'blue' },
})

Box.globalFlair = flair({
  '.title': { color: 'red' },
})
