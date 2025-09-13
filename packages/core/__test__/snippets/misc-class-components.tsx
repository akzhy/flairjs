import { Style } from '@flairjs/react'
import clsx from 'clsx'
import { Component } from 'react'

class Box extends Component<{
  children?: React.ReactNode
  containerClassName?: string
}> {
  render() {
    const { children, containerClassName } = this.props

    return (
      <div className={clsx('box', containerClassName)}>
        {children}
        <span className="title">Hello</span>
        <Style>
          {
            /*css*/ `
        .box {
          color: blue;
        }`
          }
        </Style>
      </div>
    )
  }
}

export { Box }
