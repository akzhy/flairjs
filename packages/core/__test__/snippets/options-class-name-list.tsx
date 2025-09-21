import clsx from 'clsx'
import { flair } from '@flairjs/client'

export const TestCaseComponent = () => {
  return (
    <>
      <div>
        <Box containerClassName="item"></Box>
        <RegexBox classList={['regex-box']} />
      </div>
    </>
  )
}

const Box = ({ children, containerClassName }: { children?: React.ReactNode; containerClassName?: string }) => {
  return <div className={clsx('box', containerClassName)}>{children}</div>
}

const RegexBox = ({ children, classList }: { children?: React.ReactNode; classList?: string[] }) => {
  return <div className={clsx('regex-box', classList)}>{children}</div>
}

Box.flair = flair({
  '.box': {
    padding: '10px',
    border: '1px solid black',
  },
})

TestCaseComponent.flair = flair({
  '.item': {
    width: '200px',
    height: '200px',
  },
  '.regex-box': {
    width: '100px',
    height: '100px',
    backgroundColor: 'lightblue',
  },
})
