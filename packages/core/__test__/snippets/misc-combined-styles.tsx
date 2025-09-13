import clsx from 'clsx'
import { flair, Style } from '@flairjs/react'

const Box = ({ children, containerClassName }: { children?: React.ReactNode; containerClassName?: string }) => {
  return (
    <div className={clsx('box', containerClassName)}>
      {children}
      <span className="title">Hello</span>
      <Style>
        {
          /*css*/ `
        .title {
          color: blue;
        }`
        }
      </Style>
    </div>
  )
}

const SecondBox = ({ children, classList }: { children?: React.ReactNode; classList?: string[] }) => {
  return <div className={clsx('box', classList)}>{children}</div>
}

Box.flair = flair({
  '.box': {
    padding: '10px',
    border: '1px solid black',
  },
})

SecondBox.flair = flair({
  '.box': {
    marginTop: '10px',
    border: '2px dashed green',
  },
})
