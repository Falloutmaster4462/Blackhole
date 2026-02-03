/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        blackhole: {
          50: '#f6f6f7',
          100: '#e1e3e5',
          200: '#c3c7cc',
          300: '#9da3ab',
          400: '#797f8a',
          500: '#5f646f',
          600: '#4c5059',
          700: '#3f424a',
          800: '#36383f',
          900: '#2f3137',
          950: '#18191d',
        },
      },
      keyframes: {
        'fade-in': {
          '0%': { opacity: '0', transform: 'translateY(-10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'slide-in': {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(0)' },
        },
      },
      animation: {
        'fade-in': 'fade-in 0.2s ease-out',
        'slide-in': 'slide-in 0.3s ease-out',
      },
    },
  },
  plugins: [],
  darkMode: 'class',
}
