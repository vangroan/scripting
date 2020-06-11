#version 150

in vec4 a_Pos;
in vec3 a_Color;

uniform mat4 u_Model;

out vec4 v_Color;

void main() {
    v_Color = vec4(a_Color, 1.0);
    gl_Position = a_Pos * u_Model;
}
