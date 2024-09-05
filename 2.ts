function onSetCheckedRecursive(items: AuthResItem[], checked: boolean): void {
    items.forEach((item) => {
        item.checked = checked // 设置当前节点的 checked 属性

        // 如果有子节点，则递归处理子节点
        if (item.children && item.children.length > 0) {
            onSetCheckedRecursive(item.children, checked)
        }
        if (item.btnList && item.btnList.length > 0) {
            onSetCheckedRecursive(item.btnList, checked)
        }
    })
}

// 选中所有行的处理函数
const selectedRes = ref<AuthResItem[]>()
function onSelectAllChange(selectedRows: AuthResItem[]) {
    setTimeout(()=>{
        selectedRes.value = selectedRows
        if (selectedRows.length) {
            onSetCheckedRecursive(resData.value!, true)
        } else {
            onSetCheckedRecursive(resData.value!, false)
        }
    },100)

}

function onSelectionChange(selection: AuthResItem) {
    console.log(selection, '----selection')
    // 先取消所有节点的 checked 状态
    onSetCheckedRecursive(resData.value!, false)

    // 递归处理选中的节点及其子节点
    selection.forEach((selectedNode: AuthResItem) => {
        // 根据 selectedNode 更新其在数据中的对应节点
        findAndUpdateNode(resData.value!, selectedNode, true)
    })
}

// 找到并更新节点
function findAndUpdateNode(nodes: AuthResItem[], targetNode: AuthResItem, checked: boolean) {
    for (const node of nodes) {
        if (node === targetNode) {
            node.checked = checked
            onSetCheckedRecursive(node.children || [], checked)
            onSetCheckedRecursive(node.btnList || [], checked)
            return
        }
        if (node.children) {
            findAndUpdateNode(node.children, targetNode, checked)
        }
        if (node.btnList) {
            findAndUpdateNode(node.btnList, targetNode, checked)
        }
    }
}