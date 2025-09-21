import { flair } from '@flairjs/client'
import { Style } from '@flairjs/client/react'
import clsx from 'clsx'
import { forwardRef } from 'react'

export const Box = ({ children, containerClassName }: { children?: React.ReactNode; containerClassName?: string }) => {
  return (
    <div className={clsx('box', containerClassName)}>
      {children}
      <span className="title">Hello</span>
    </div>
  )
}

export function Box2({ children, containerClassName }: { children?: React.ReactNode; containerClassName?: string }) {
  return (
    <div className={clsx('box', containerClassName)}>
      {children}
      <span className="title">Hello</span>
    </div>
  )
}

export const Box3: React.FC<{ children?: React.ReactNode; containerClassName?: string }> = ({
  children,
  containerClassName,
}) => {
  return (
    <div className={clsx('box', containerClassName)}>
      {children}
      <span className="title">Hello</span>
    </div>
  )
}

export const Box4 = function ({
  children,
  containerClassName,
}: {
  children?: React.ReactNode
  containerClassName?: string
}) {
  return (
    <div className={clsx('box', containerClassName)}>
      {children}
      <span className="title">Hello</span>
    </div>
  )
}

export const Box5 = forwardRef<HTMLDivElement, { children?: React.ReactNode; containerClassName?: string }>(
  ({ children, containerClassName }, ref) => {
    return (
      <div className={clsx('box', containerClassName)} ref={ref}>
        {children}
        <span className="title">Hello</span>
      </div>
    )
  },
)

export const Box6 = forwardRef<HTMLDivElement, { children?: React.ReactNode; containerClassName?: string }>(
  ({ children, containerClassName }, ref) => {
    return (
      <div className={clsx('box', containerClassName)} ref={ref}>
        {children}
        <span className="title">Hello</span>
        <Style>
          {
            /*css*/ `.box {
            color: blue;
          }
          `
          }
        </Style>
      </div>
    )
  },
)

Box.flair = flair({
  '.box': { color: 'blue' },
})

Box2.flair = flair({
  '.box': { color: 'red' },
})

// @ts-ignore
Box3.flair = flair({
  '.box': { color: 'orange' },
})

Box4.flair = flair({
  '.box': { color: 'green' },
})

// @ts-ignore
Box5.flair = flair({
  '.box': { color: 'black' },
})
