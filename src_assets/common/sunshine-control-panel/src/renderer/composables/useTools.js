import { ElMessage, ElMessageBox } from 'element-plus'
import { openExternalUrl, tools } from '@/tauri-adapter.js'

/**
 * 工具操作 Composable
 */
export function useTools() {
  /**
   * 公共确认对话框操作
   * @param {string} message - 确认消息
   * @param {string} title - 对话框标题
   * @param {function} action - 执行的操作
   * @param {string} successMsg - 成功消息
   */
  const confirmAction = async (message, title, action, successMsg) => {
    try {
      await ElMessageBox.confirm(message, title, {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      })
      await action()
      ElMessage.success(successMsg)
    } catch (error) {
      if (error !== 'cancel') {
        ElMessage.error(`操作失败: ${error}`)
      }
    }
  }

  /**
   * 卸载 VDD
   */
  const uninstallVdd = async () => {
    await confirmAction(
      '确定要卸载虚拟显示器驱动吗？此操作需要管理员权限。',
      '确认卸载',
      tools.uninstallVddDriver,
      '卸载请求已发送'
    )
  }

  /**
   * 重启显卡驱动
   */
  const restartDriver = async () => {
    await confirmAction(
      '确定要重启显卡驱动吗？这将暂时中断屏幕显示。',
      '确认重启',
      tools.restartGraphicsDriver,
      '重启请求已发送'
    )
  }

  /**
   * 重启 Sunshine 服务
   */
  const restartSunshine = async () => {
    await confirmAction(
      '确定要重启 Sunshine 服务吗？这将断开当前所有连接。\n\n如果弹出 UAC 提示，请点击"是"以确认。\nSunshine 服务将在几秒钟内重启。',
      '确认重启',
      tools.restartSunshineService,
      '重启请求已发送'
    )
  }

  /**
   * 打开串流计时器
   */
  const openTimer = async () => {
    await createWindow('/stop-clock-canvas/index.html', '串流计时器', {
      prefix: 'timer',
      width: 1080,
      height: 600,
    })
  }

  /**
   * 打开延迟测试工具（嵌入式窗口）
   */
  const openDelayTester = async () => {
    await createWindow('https://yangkile.github.io/D-lay/', '延迟测试', {
      prefix: 'delay',
      width: 900,
      height: 700,
    })
  }

  /**
   * 打开手柄测试工具（嵌入式窗口）
   */
  const openGamepadTester = async () => {
    await createWindow('https://hardwaretester.com/gamepad', '手柄测试', {
      prefix: 'gamepad',
      width: 1000,
      height: 750,
    })
  }

  /**
   * 打开外部 URL
   * @param {string} url - 要打开的URL
   */
  const openUrl = async (url) => {
    await openExternalUrl(url)
  }

  /**
   * 清理无用的封面图片和临时文件
   */
  const cleanupCovers = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')

      // 首先检查是否以管理员权限运行
      const isRunningAsAdmin = await invoke('is_running_as_admin')

      if (!isRunningAsAdmin) {
        // 不是管理员，提示重启
        await ElMessageBox.confirm('清理临时文件需要管理员权限。\n\n是否以管理员身份重启应用？', '需要管理员权限', {
          confirmButtonText: '以管理员重启',
          cancelButtonText: '取消',
          type: 'warning',
        })

        // 用户确认后，调用重启为管理员
        await restartAsAdmin()
        return
      }

      // 已经是管理员，继续执行清理
      await ElMessageBox.confirm(
        '此操作将删除：\n1. 未被应用使用的封面图片\n2. config 目录下的 temp_ 临时文件\n\n是否继续？',
        '清理无用文件',
        {
          confirmButtonText: '确定',
          cancelButtonText: '取消',
          type: 'warning',
        }
      )

      // 显示加载提示
      const loading = ElMessage({
        message: '正在清理无用文件...',
        type: 'info',
        duration: 0,
      })

      // 调用 Tauri 命令
      const result = await invoke('cleanup_unused_covers')

      loading.close()

      // 显示结果
      if (result.success) {
        if (result.deleted_count > 0) {
          ElMessageBox.alert(
            `${result.message}\n\n删除的文件数: ${result.deleted_count}\n释放的空间: ${(
              result.freed_space / 1024
            ).toFixed(2)} KB`,
            '清理完成',
            {
              confirmButtonText: '确定',
              type: 'success',
            }
          )
        } else {
          ElMessage.success(result.message)
        }
      } else {
        ElMessage.error('清理失败: ' + result.message)
      }
    } catch (error) {
      if (error !== 'cancel') {
        console.error('清理文件失败:', error)
        ElMessage.error('清理文件失败: ' + error)
      }
    }
  }

  /**
   * 以管理员权限重启 GUI
   */
  const restartAsAdmin = async () => {
    try {
      // 确认对话框
      await ElMessageBox.confirm('将以管理员权限重启应用，当前窗口会关闭。是否继续？', '提升权限', {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      })

      // 显示提示
      ElMessage.info('正在请求管理员权限...')

      // 调用 Tauri 命令
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('restart_as_admin')

      // 如果到这里说明成功请求了重启
      ElMessage.success('正在以管理员权限重启...')
    } catch (error) {
      if (error !== 'cancel') {
        console.error('重启失败:', error)
        ElMessage.error('重启失败: ' + error)
      }
    }
  }

  /**
   * 检查更新
   */
  const checkForUpdates = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')

      ElMessage.info('正在检查更新...')

      const result = await invoke('check_for_updates')

      if (result) {
        return result // 返回更新信息，让调用者处理
      } else {
        ElMessage.success('已是最新版本')
        return null
      }
    } catch (error) {
      if (error.includes('已是最新版本')) {
        ElMessage.success('已是最新版本')
      } else {
        console.error('检查更新失败:', error)
        ElMessage.error('检查更新失败: ' + error)
      }
      return null
    }
  }



  /**
   * 打开剪贴板同步工具（系统弹窗风格）
   */
  const openClipboardSync = async () => {
    try {
      await ElMessageBox.confirm(
        '请选择剪贴板同步操作：\n\n- 推送：将本地剪贴板内容上传到云端\n- 拉取：将云端内容覆盖到本地剪贴板',
        '剪贴板云同步',
        {
          confirmButtonText: '推送到云端 (Upload)',
          cancelButtonText: '从云端拉取 (Download)',
          distinguishCancelAndClose: true,
          type: 'info',
          center: true,
        }
      )

      // 用户点击了 Confirm (Upload)
      await handleClipboardSync('upload')
    } catch (action) {
      if (action === 'cancel') {
        // 用户点击了 Cancel (Download)
        await handleClipboardSync('download')
      }
      // 其他情况（点击关闭X或ESC）不做处理
    }
  }

  /**
   * 处理剪贴板同步的具体逻辑
   * @param {string} type - 'upload' | 'download'
   */
  const handleClipboardSync = async (type) => {
    const loading = ElMessage({
      message: type === 'upload' ? '正在推送到云端...' : '正在从云端拉取...',
      type: 'info',
      duration: 0,
    })

    try {
      const { invoke } = await import('@tauri-apps/api/core')

      if (type === 'upload') {
        const text = await navigator.clipboard.readText()
        if (!text) {
          throw new Error('剪贴板为空')
        }
        await invoke('galgame_sync_clipboard_to_cloud', { text })
        ElMessage.success('推送成功')
      } else {
        const text = await invoke('galgame_sync_clipboard_from_cloud')
        await navigator.clipboard.writeText(text)
        ElMessage.success('拉取成功')
      }
    } catch (error) {
      console.error('同步失败:', error)
      ElMessage.error(`同步失败: ${error}`)
    } finally {
      loading.close()
    }
  }

  /**
   * 公共窗口创建函数
   * @param {string} url - 窗口URL路径
   * @param {string} title - 窗口标题
   * @param {object} options - 窗口配置选项
   */
  const createWindow = async (url, title, options = {}) => {
    try {
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow')

      // 判断是否为外部链接
      const isExternal = url.startsWith('http://') || url.startsWith('https://')
      const baseUrl = window.location.origin
      const finalUrl = isExternal ? url : `${baseUrl}${url}`

      const windowId = `${options.prefix || 'window'}_${Date.now()}`

      const newWindow = new WebviewWindow(windowId, {
        url: finalUrl,
        title,
        width: options.width || 1080,
        height: options.height || 800,
        resizable: options.resizable !== false,
        decorations: options.decorations !== false,
        center: true,
      })

      // 等待窗口创建完成后显示
      newWindow.once('tauri://created', async () => {
        console.log(`✅ ${title}窗口已创建`)
        await newWindow.show()
        await newWindow.setFocus()
        console.log(`✅ ${title}窗口已显示`)
      })

      newWindow.once('tauri://error', (e) => {
        console.error(`❌ ${title}窗口创建失败:`, e)
        ElMessage.error(`${title}窗口创建失败`)
      })
    } catch (error) {
      console.error(`❌ 打开${title}失败:`, error)
      ElMessage.error(`打开${title}失败: ${error.message}`)
    }
  }

  return {
    confirmAction,
    uninstallVdd,
    restartDriver,
    restartSunshine,
    openTimer,
    openDelayTester,
    openGamepadTester,
    openClipboardSync,
    openUrl,
    cleanupCovers,
    restartAsAdmin,
    checkForUpdates,
    createWindow,
  }
}


