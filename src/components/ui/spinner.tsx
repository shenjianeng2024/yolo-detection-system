"use client"

import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

const spinnerVariants = cva(
  "animate-spin rounded-full border-solid border-t-transparent",
  {
    variants: {
      size: {
        small: "h-4 w-4 border-2",
        medium: "h-8 w-8 border-2",
        large: "h-12 w-12 border-3",
      },
      variant: {
        default: "border-primary",
        secondary: "border-secondary-foreground",
        white: "border-white",
      },
    },
    defaultVariants: {
      size: "medium",
      variant: "default",
    },
  }
)

interface SpinnerProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof spinnerVariants> {}

const Spinner = React.forwardRef<HTMLDivElement, SpinnerProps>(
  ({ className, size, variant, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(spinnerVariants({ size, variant }), className)}
        {...props}
      />
    )
  }
)
Spinner.displayName = "Spinner"

export { Spinner, spinnerVariants }