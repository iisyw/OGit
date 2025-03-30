@echo off
:: 设置命令行使用 UTF-8 编码
chcp 65001 >nul

:: 执行PushGit工具并传递所有参数
.\pushgit.exe %*

:: 脚本结束
exit /b %errorlevel%
