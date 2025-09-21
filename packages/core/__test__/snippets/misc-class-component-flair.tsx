import { flair } from '@flairjs/client'
import clsx from 'clsx'
import { Component } from 'react'

class Box extends Component<{
  children?: React.ReactNode
  containerClassName?: string
}> {
  render() {
    const { containerClassName, children } = this.props
    return (
      <div className={clsx('box', containerClassName)}>
        {children}
        <span className="title">Hello</span>
      </div>
    )
  }
}

// @ts-ignore
Box.flair = flair({
  '.box': { color: 'blue' },
})
