/** @type {import('tailwindcss').Config} */
export default {
  content: [],
  theme: {
    extend: {
      colors: {
        primary: 'var(--color-primary)',
        secondary: 'var(--color-secondary)',
        accent: 'var(--color-accent)',
        neutral: 'var(--color-neutral)',
        background: 'var(--color-background)',
        outline: 'var(--color-outline)',
        cardbackground: 'var(--color-card-background)',
        text: 'var(--color-text)',
      },

      borderRadius: {
        default: 'var(--border-radius)',
      },


      width: {
        content: 'var(--content-width)',
        sidebar: 'var(--sidebar-width)',
      },
      minWidth: {
        content: 'var(--content-width)',
        sidebar: 'var(--sidebar-width)',
      },
      maxWidth: {
        content: 'var(--content-width)',
        sidebar: 'var(--sidebar-width)',
      },

      keyframes: {
        slideDown: {
          from: { height: 0 },
          to: { height: 'var(--radix-accordion-content-height)' },
        },
        slideUp: {
          from: { height: 'var(--radix-accordion-content-height)' },
          to: { height: 0 },
        },
      },
      animation: {
        slideDown: 'slideDown 300ms cubic-bezier(0.87, 0, 0.13, 1)',
        slideUp: 'slideUp 300ms cubic-bezier(0.87, 0, 0.13, 1)',
      },
    },
  },
  plugins: [],
}

