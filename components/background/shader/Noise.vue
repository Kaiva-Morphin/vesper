<template>
  <div class=" h-full w-full top-0 left-0 fixed">
    <div ref="container" class="absolute top-0 shader-background"/>
    <div class="absolute top-0 h-full w-full glass"></div>
  </div>
</template>
<style scoped>
.glass {
  background: rgba(0, 0, 0, 0.0);
  backdrop-filter: blur(4px); 
  
  transition: background-color var(--interpolation-speed) var(--interpolation-method);

  background-color: var(--color-primary);
  mix-blend-mode: overlay;
}
.shader-background {
  width: 100%;
  height: 100%;
  position: absolute;
  top: 0;
  left: 0;
  overflow: hidden;
}
</style>
<script>
export default {
  async mounted() {
    const fragmentShaderSource = await this.loadShaderSource('/shaders/background.glsl');
    const colorPrimary = getComputedStyle(document.documentElement).getPropertyValue('--color-primary').trim();
    this.initShader(fragmentShaderSource, colorPrimary);
  },
  data() {
    return {
      u_colorPrimary_location: null,
      gl_context: null
    }
  },
  methods: {
    async loadShaderSource(url) {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Failed to load shader source: ${response.statusText}`);
      }
      return response.text();
    },
    setColor(hex) {
      const colorPrimaryRGB = this.hexToRgb(hex);
      this.gl_context.uniform3fv(this.u_colorPrimary_location, colorPrimaryRGB);
    },
    hexToRgb(hex) {
      const bigint = parseInt(hex.slice(1), 16);
      const r = (bigint >> 16) & 255;
      const g = (bigint >> 8) & 255;
      const b = bigint & 255;
      return [r / 255, g / 255, b / 255];
    },
    initShader(fragmentShaderSource, colorPrimary) {
      const container = this.$refs.container;
      const canvas = document.createElement('canvas');
      container.appendChild(canvas);
      const gl = canvas.getContext('webgl');
      this.gl_context = gl;
      const vertexShaderSource = `
        attribute vec4 a_position;
        void main() {
          gl_Position = a_position;
        }
      `;

      const vertexShader = this.createShader(gl, gl.VERTEX_SHADER, vertexShaderSource);
      const fragmentShader = this.createShader(gl, gl.FRAGMENT_SHADER, fragmentShaderSource);
      const program = this.createProgram(gl, vertexShader, fragmentShader);

      const positionAttributeLocation = gl.getAttribLocation(program, "a_position");
      const resolutionUniformLocation = gl.getUniformLocation(program, "u_resolution");
      const mouseUniformLocation = gl.getUniformLocation(program, "u_mouse");
      const timeUniformLocation = gl.getUniformLocation(program, "u_time");

      const positionBuffer = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
      const positions = [
        -1, -1,
         1, -1,
        -1,  1,
        -1,  1,
         1, -1,
         1,  1,
      ];
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

      gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);

      gl.useProgram(program);
      const colorPrimaryLocation = gl.getUniformLocation(program, 'u_colorPrimary');
      this.u_colorPrimary_location = colorPrimaryLocation;
      const colorPrimaryRGB = this.hexToRgb(colorPrimary);
      gl.uniform3fv(colorPrimaryLocation, colorPrimaryRGB);

      gl.enableVertexAttribArray(positionAttributeLocation);
      gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
      gl.vertexAttribPointer(positionAttributeLocation, 2, gl.FLOAT, false, 0, 0);

      const startTime = Date.now();

      var glob_x = -9000, glob_y = -9000;
      var target_x = 0, target_y = 0;
      var interpolated_x = 0, interpolated_y = 0;

      /*
      document.onmousemove = e => {
        target_x = e.clientX;
        target_y = e.clientY;
      };
      */


      window.addEventListener('resize', () => {
        canvas.width = container.clientWidth;
        canvas.height = container.clientHeight;
        gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
      });

      canvas.width = container.clientWidth;
      canvas.height = container.clientHeight;
      gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);

      const animate = () => {
        interpolated_x += (target_x - interpolated_x) * 0.1;
        interpolated_y += (target_y - interpolated_y) * 0.1;
        if (Math.abs(interpolated_x - glob_x) > 0.1 || Math.abs(interpolated_y - glob_y) > 0.1) {
          glob_x = interpolated_x;
          glob_y = interpolated_y;
        }
        const currentTime = (Date.now() - startTime) / 1000;
        gl.uniform1f(timeUniformLocation, currentTime);
        gl.uniform2f(mouseUniformLocation, glob_x, glob_y);
        gl.uniform2f(resolutionUniformLocation, gl.canvas.width, gl.canvas.height);
        gl.drawArrays(gl.TRIANGLES, 0, 6);
        requestAnimationFrame(animate);
      };

      animate();
    },
    createShader(gl, type, source) {
      const shader = gl.createShader(type);
      gl.shaderSource(shader, source);
      gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error(gl.getShaderInfoLog(shader));
        gl.deleteShader(shader);
        return null;
      }
      return shader;
    },
    createProgram(gl, vertexShader, fragmentShader) {
      const program = gl.createProgram();
      gl.attachShader(program, vertexShader);
      gl.attachShader(program, fragmentShader);
      gl.linkProgram(program);
      if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        console.error(gl.getProgramInfoLog(program));
        gl.deleteProgram(program);
        return null;
      }
      return program;
    }
  }
}
</script>

