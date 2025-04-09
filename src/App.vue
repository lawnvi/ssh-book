<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

// 数据结构
interface Server {
  id: string;
  name: string;
  username: string;
  host: string;
  port: number;
  auth_type: 'password' | 'key';
  auth_info: string;
  group_id: string;
}

interface Group {
  id: string;
  name: string;
}

// 状态管理
const groups = ref<Group[]>([]);
const servers = ref<Server[]>([]);
const currentGroup = ref<string>('');
const showAddGroup = ref(false);
const newGroupName = ref('');
const showAddServerDialog = ref(false);
const showEditServerDialog = ref(false);
const editingGroup = ref<Group | null>(null);
const editingServer = ref<Server | null>(null);
const showImportDialog = ref(false);
const importData = ref('');
const searchQuery = ref('');

const newServer = ref({
  name: '',
  username: '',
  host: '',
  port: 22,
  authType: 'password' as 'password' | 'key',
  authInfo: '~/.ssh/id_rsa',
  groupId: ''
});

// 计算属性：过滤后的服务器列表
const filteredServers = computed(() => {
  if (!searchQuery.value) {
    return servers.value.filter(s => s.group_id === currentGroup.value);
  }
  
  const query = searchQuery.value.toLowerCase();
  return servers.value.filter(server => 
    server.name.toLowerCase().includes(query) || 
    server.host.toLowerCase().includes(query)
  );
});

// 加载数据
async function loadData() {
  try {
    console.log('Loading data...');
    const [groupsData, serversData] = await Promise.all([
      invoke('get_groups') as Promise<Group[]>,
      invoke('get_servers') as Promise<Server[]>
    ]);
    groups.value = groupsData;
    servers.value = serversData;
    console.log('Data loaded successfully');
  } catch (error) {
    console.error('Failed to load data:', error);
    alert('加载数据失败：' + error);
  }
}

// 添加分组
async function addGroup() {
  if (!newGroupName.value) return;
  try {
    await invoke('add_group', { name: newGroupName.value });
    newGroupName.value = '';
    showAddGroup.value = false;
    await loadData();
  } catch (error) {
    console.error('Failed to add group:', error);
  }
}

// 更新分组
async function updateGroup(group: Group) {
  try {
    await invoke('update_group', { group });
    editingGroup.value = null;
    await loadData();
  } catch (error) {
    console.error('Failed to update group:', error);
  }
}

// 删除分组
async function deleteGroup(id: string, name: string) {
  if (!await confirm(`确定要删除${name}吗？`)) return;
  try {
    await invoke('delete_group', { id });
    await loadData();
  } catch (error) {
    console.error('Failed to delete group:', error);
    alert('删除分组失败：' + error);
  }
}

// 添加服务器
async function addServer() {
  if (!currentGroup.value) {
    alert('请先选择一个分组');
    return;
  }
  
  if (!newServer.value.name) {
    alert('请输入服务器名称');
    return;
  }
  
  if (!newServer.value.username) {
    alert('请输入用户名');
    return;
  }
  
  if (!newServer.value.host) {
    alert('请输入主机地址');
    return;
  }
  
  if (newServer.value.authType === 'key' && !newServer.value.authInfo) {
    newServer.value.authInfo = '~/.ssh/id_rsa';
  }

  try {
    const serverToAdd: Server = {
      id: crypto.randomUUID(),
      name: newServer.value.name,
      username: newServer.value.username,
      host: newServer.value.host,
      port: newServer.value.port || 22,
      auth_type: newServer.value.authType,
      auth_info: newServer.value.authInfo,
      group_id: currentGroup.value
    };
    
    console.log('Adding server:', serverToAdd);
    await invoke('add_server', { server: serverToAdd });
    showAddServerDialog.value = false;
    newServer.value = {
      name: '',
      username: '',
      host: '',
      port: 22,
      authType: 'password',
      authInfo: '~/.ssh/id_rsa',
      groupId: currentGroup.value
    };
    await loadData();
  } catch (error) {
    console.error('Failed to add server:', error);
    alert('添加服务器失败：' + error);
  }
}

// 更新服务器
async function updateServer(server: Server) {
  try {
    const serverToUpdate = {
      ...server,
      auth_type: server.auth_type,
      auth_info: server.auth_info,
      group_id: server.group_id
    };
    await invoke('update_server', { server: serverToUpdate });
    editingServer.value = null;
    showEditServerDialog.value = false;
    await loadData();
  } catch (error) {
    console.error('Failed to update server:', error);
    alert('更新服务器失败：' + error);
  }
}

// 开始编辑服务器
function startEditServer(server: Server) {
  editingServer.value = { 
    ...server,
    port: server.port || 22,
    auth_info: server.auth_type === 'key' ? (server.auth_info || '~/.ssh/id_rsa') : server.auth_info
  };
  showEditServerDialog.value = true;
}

// 删除服务器
async function deleteServer(id: string, name: string) {
  if (!await confirm(`确定要删除${name}吗？`)) return;
  try {
    await invoke('delete_server', { id });
    await loadData();
  } catch (error) {
    console.error('Failed to delete server:', error);
    alert('删除服务器失败：' + error);
  }
}

// 连接服务器
async function connectServer(server: Server) {
  try {
    await invoke('connect_server', { server });
  } catch (error) {
    console.error('Failed to connect to server:', error);
  }
}

// 导出数据
async function exportConfig() {
  try {
    const result = await invoke('export_data');
    alert(result);
  } catch (error) {
    console.error('Failed to export data:', error);
    alert('导出配置失败：' + error);
  }
}

// 导入数据
async function importConfig() {
  try {
    const result = await invoke('import_data');
    await loadData();
    alert(result);
  } catch (error) {
    console.error('Failed to import data:', error);
    alert('导入配置失败：' + error);
  }
}

// 初始化
onMounted(() => {
  loadData();
});
</script>

<template>
  <div class="app-container">
    <!-- 侧边栏 -->
    <div class="sidebar">
      <div class="sidebar-header">
        <h2>分组</h2>
        <div class="header-actions">
          <!-- <button class="icon-button" @click="exportConfig" title="导出配置">
            <span class="icon">↓</span>
          </button>
          <button class="icon-button" @click="importConfig" title="导入配置">
            <span class="icon">↑</span>
          </button> -->
          <button class="icon-button" @click="showAddGroup = true" title="添加分组">
            <span class="icon">+</span>
          </button>
        </div>
      </div>
      
      <!-- 搜索框 -->
      <div class="search-box">
        <input 
          v-model="searchQuery"
          placeholder="搜索服务器或地址"
          class="search-input"
        />
      </div>
      
      <div class="group-list">
        <div v-if="showAddGroup" class="group-item editing">
          <input 
            v-model="newGroupName" 
            placeholder="输入分组名称" 
            @keyup.enter="addGroup"
            @blur="addGroup"
            autofocus
          />
        </div>
        
        <div 
          v-for="group in groups" 
          :key="group.id"
          class="group-item"
          :class="{ active: currentGroup === group.id }"
        >
          <div class="group-content" @click="currentGroup = group.id">
            <span v-if="editingGroup?.id !== group.id">{{ group.name }}</span>
            <input 
              v-else
              v-model="editingGroup.name"
              @keyup.enter="updateGroup(editingGroup)"
              @blur="updateGroup(editingGroup)"
              autofocus
            />
          </div>
          <div class="group-actions">
            <button class="icon-button" @click.stop="editingGroup = { ...group }">
              <span class="icon">✎</span>
            </button>
            <button class="icon-button danger" @click.stop="deleteGroup(group.id, group.name)">
              <span class="icon">×</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="main-content">
      <div v-if="currentGroup || searchQuery" class="server-list">
        <div 
          v-for="server in filteredServers" 
          :key="server.id"
          class="server-item"
        >
          <div class="server-info">
            <h3>{{ server.name }}</h3>
            <p>{{ server.username }}@{{ server.host }}</p>
            <p v-if="searchQuery" class="server-group">
              分组：{{ groups.find(g => g.id === server.group_id)?.name }}
            </p>
          </div>
          <div class="server-actions">
            <button class="icon-button" @click.stop="connectServer(server)">
              <span class="icon">→</span>
            </button>
            <button class="icon-button" @click.stop="startEditServer(server)">
              <span class="icon">✎</span>
            </button>
            <button class="icon-button danger" @click.stop="deleteServer(server.id, server.name)">
              <span class="icon">×</span>
            </button>
          </div>
        </div>
      </div>
      <div v-else class="empty-state">
        <p>请选择一个分组或创建新分组</p>
      </div>
    </div>

    <!-- 添加服务器按钮 -->
    <button 
      v-if="currentGroup"
      class="floating-button"
      @click="showAddServerDialog = true"
    >
      <span class="icon">+</span>
    </button>

    <!-- 添加服务器对话框 -->
    <div v-if="showAddServerDialog" class="dialog-overlay">
      <div class="dialog">
        <!-- <h3>添加服务器</h3> -->
        <div class="form-group">
          <div class="form-item">
            <label>服务器名称</label>
            <input v-model="newServer.name" placeholder="请输入服务器名称" />
          </div>
          <div class="form-item">
            <label>用户名</label>
            <input v-model="newServer.username" placeholder="请输入用户名" />
          </div>
          <div class="form-item">
            <label>主机地址</label>
            <input v-model="newServer.host" placeholder="请输入主机地址" />
          </div>
          <div class="form-item">
            <label>端口</label>
            <input 
              v-model.number="newServer.port" 
              type="number" 
              min="1" 
              max="65535" 
              placeholder="请输入端口号" 
            />
          </div>
          <div class="form-item">
            <label>认证方式</label>
            <select v-model="newServer.authType">
              <option value="password">密码登录</option>
              <option value="key">密钥登录</option>
            </select>
          </div>
          <div class="form-item">
            <label>{{ newServer.authType === 'password' ? '密码' : '密钥路径' }}</label>
            <input 
              v-model="newServer.authInfo" 
              :placeholder="newServer.authType === 'password' ? '请输入密码' : '请输入密钥路径'" 
              :type="newServer.authType === 'password' ? 'password' : 'text'"
            />
          </div>
        </div>
        <div class="dialog-actions">
          <button @click="showAddServerDialog = false">取消</button>
          <button @click="addServer">确定</button>
        </div>
      </div>
    </div>

    <!-- 编辑服务器对话框 -->
    <div v-if="showEditServerDialog && editingServer" class="dialog-overlay">
      <div class="dialog">
        <!-- <h3>编辑服务器</h3> -->
        <div class="form-group">
          <div class="form-item">
            <label>服务器名称</label>
            <input v-model="editingServer.name" placeholder="请输入服务器名称" />
          </div>
          <div class="form-item">
            <label>用户名</label>
            <input v-model="editingServer.username" placeholder="请输入用户名" />
          </div>
          <div class="form-item">
            <label>主机地址</label>
            <input v-model="editingServer.host" placeholder="请输入主机地址" />
          </div>
          <div class="form-item">
            <label>端口</label>
            <input 
              v-model.number="editingServer.port" 
              type="number" 
              min="1" 
              max="65535" 
              placeholder="请输入端口号" 
            />
          </div>
          <div class="form-item">
            <label>认证方式</label>
            <select v-model="editingServer.auth_type">
              <option value="password">密码登录</option>
              <option value="key">密钥登录</option>
            </select>
          </div>
          <div class="form-item">
            <label>{{ editingServer.auth_type === 'password' ? '密码' : '密钥路径' }}</label>
            <input 
              v-model="editingServer.auth_info" 
              :placeholder="editingServer.auth_type === 'password' ? '请输入密码' : '请输入密钥路径'" 
              :type="editingServer.auth_type === 'password' ? 'password' : 'text'"
            />
          </div>
        </div>
        <div class="dialog-actions">
          <button @click="showEditServerDialog = false">取消</button>
          <button @click="updateServer(editingServer)">确定</button>
        </div>
      </div>
    </div>

    <!-- 导入配置对话框 -->
    <div v-if="showImportDialog" class="dialog-overlay">
      <div class="dialog">
        <h3>导入配置</h3>
        <div class="form-group">
          <div class="form-item">
            <label>配置内容</label>
            <textarea 
              v-model="importData" 
              placeholder="请粘贴配置内容" 
              rows="10"
            ></textarea>
          </div>
        </div>
        <div class="dialog-actions">
          <button @click="showImportDialog = false">取消</button>
          <button @click="importConfig">确定</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
/* 全局样式 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  margin: 0;
  padding: 0;
  background-color: var(--bg-color);
  color: var(--text-color);
}

:root {
  --primary-color: #007AFF;
  --text-color: #333;
  --bg-color: #fff;
  --border-color: #ddd;
  --hover-color: #f5f5f5;
  --danger-color: #ff3b30;
}

.app-container {
  display: flex;
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  color: var(--text-color);
  background-color: var(--bg-color);
}

/* 侧边栏样式 */
.sidebar {
  width: 250px;
  background-color: var(--bg-color);
  padding: 20px;
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.sidebar-header h2 {
  margin: 0;
  font-size: 1.2em;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.group-list {
  flex: 1;
  overflow-y: auto;
}

.group-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  border-radius: 8px;
  margin-bottom: 5px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.group-item:hover {
  background-color: var(--hover-color);
}

.group-item.active {
  background-color: var(--primary-color);
  color: white;
}

.group-content {
  flex: 1;
}

.group-actions {
  display: flex;
  gap: 5px;
  opacity: 0;
  transition: opacity 0.2s;
}

.group-item:hover .group-actions {
  opacity: 1;
}

.group-actions button {
  background: none;
  border: none;
  padding: 5px;
  cursor: pointer;
  color: inherit;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.group-actions button:hover {
  background-color: rgba(0, 0, 0, 0.1);
}

.group-actions button.danger {
  color: var(--danger-color);
}

.group-actions button.danger:hover {
  background-color: rgba(255, 59, 48, 0.1);
}

/* 主内容区样式 */
.main-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  background-color: var(--bg-color);
}

.server-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.server-item {
  background-color: var(--bg-color);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 15px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transition: transform 0.2s, box-shadow 0.2s;
}

.server-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

.server-info h3 {
  margin: 0;
  font-size: 1.1em;
}

.server-info p {
  margin: 5px 0;
  color: #666;
  font-size: 0.9em;
}

.server-actions {
  display: flex;
  gap: 5px;
  opacity: 0;
  transition: opacity 0.2s;
}

.server-item:hover .server-actions {
  opacity: 1;
}

/* 按钮样式 */
.icon-button {
  background: none;
  border: none;
  padding: 5px;
  cursor: pointer;
  color: inherit;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.icon-button:hover {
  background-color: rgba(0, 0, 0, 0.1);
}

.icon {
  font-size: 1.2em;
  line-height: 1;
}

.floating-button {
  position: fixed;
  right: 30px;
  bottom: 30px;
  width: 50px;
  height: 50px;
  border-radius: 50%;
  background-color: var(--primary-color);
  color: white;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
  transition: transform 0.2s;
}

.floating-button:hover {
  transform: scale(1.1);
}

/* 对话框样式 */
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(5px);
}

.dialog {
  background-color: var(--bg-color);
  border-radius: 16px;
  padding: 24px;
  width: 480px;
  max-width: 90%;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  animation: dialog-enter 0.3s ease-out;
}

@keyframes dialog-enter {
  from {
    opacity: 0;
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.dialog h3 {
  margin: 0 0 24px 0;
  font-size: 1.5em;
  font-weight: 600;
  color: var(--text-color);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: var(--text-color);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}

.dialog-actions button {
  padding: 8px 16px;
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.2s;
}

.dialog-actions button:first-child {
  background-color: transparent;
  color: var(--text-color);
  border: 1px solid var(--border-color);
}

.dialog-actions button:first-child:hover {
  background-color: var(--hover-color);
}

.dialog-actions button:last-child {
  background-color: var(--primary-color);
  color: white;
  border: none;
}

.dialog-actions button:last-child:hover {
  background-color: #0056b3;
  transform: translateY(-1px);
}

/* 表单样式 */
input, select {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 1em;
  background-color: var(--bg-color);
  color: var(--text-color);
  transition: all 0.2s;
}

input:focus, select:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
}

/* 空状态样式 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #666;
}

/* 暗色模式 */
@media (prefers-color-scheme: dark) {
  :root {
    --text-color: #fff;
    --bg-color: #1a1a1a;
    --border-color: #333;
    --hover-color: #2a2a2a;
  }

  body {
    background-color: var(--bg-color);
  }

  .app-container {
    background-color: var(--bg-color);
  }

  .sidebar {
    background-color: var(--bg-color);
    border-right-color: var(--border-color);
  }

  .main-content {
    background-color: var(--bg-color);
  }

  input, select {
    background-color: #2a2a2a;
    color: #fff;
    border-color: #444;
  }

  .server-item {
    background-color: #2a2a2a;
    border-color: #333;
  }

  .server-info p {
    color: #aaa;
  }

  .dialog {
    background-color: #2a2a2a;
    border: 1px solid #333;
  }

  button {
    background-color: var(--primary-color);
    color: white;
  }

  button:hover {
    background-color: #0056b3;
  }

  .dialog-overlay {
    background-color: rgba(0, 0, 0, 0.7);
  }

  input, select {
    background-color: #333;
    border-color: #444;
    color: #fff;
  }

  input:focus, select:focus {
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.2);
  }

  .dialog-actions button:first-child {
    border-color: #444;
  }

  .dialog-actions button:first-child:hover {
    background-color: #333;
  }
}

textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 1em;
  background-color: var(--bg-color);
  color: var(--text-color);
  transition: all 0.2s;
  resize: vertical;
  min-height: 120px;
  font-family: inherit;
}

textarea:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
}

/* 暗色模式下的文本域样式 */
@media (prefers-color-scheme: dark) {
  textarea {
    background-color: #333;
    border-color: #444;
    color: #fff;
  }

  textarea:focus {
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.2);
  }
}

/* 搜索框样式 */
.search-box {
  margin-bottom: 20px;
  padding: 0 10px;
}

.search-input {
  width: 100%;
  padding: 10px 15px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 0.9em;
  background-color: var(--bg-color);
  color: var(--text-color);
  transition: all 0.2s;
}

.search-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
}

/* 暗色模式下的搜索框样式 */
@media (prefers-color-scheme: dark) {
  .search-input {
    background-color: #2a2a2a;
    border-color: #444;
    color: #fff;
  }

  .search-input:focus {
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.2);
  }
}

/* 服务器分组信息样式 */
.server-group {
  font-size: 0.8em;
  color: #666;
  margin-top: 5px;
}

/* 暗色模式下的服务器分组信息样式 */
@media (prefers-color-scheme: dark) {
  .server-group {
    color: #aaa;
  }
}
</style>