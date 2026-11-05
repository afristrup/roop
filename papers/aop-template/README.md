# AOP template

`imsart.cls` and `imsart.sty` are unmodified files from the [official IMS AOP author package](https://github.com/vtex-soft/texsupport.ims-aop), licensed under MIT.

Build either manuscript from its folder:

```sh
TEXINPUTS=../aop-template: pdflatex -interaction=nonstopmode -halt-on-error main.tex
```
